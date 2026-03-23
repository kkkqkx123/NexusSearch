//! WAL (Write-Ahead Log) 模块
//!
//! 提供预写式日志功能，确保数据持久性和崩溃恢复能力

use crate::error::{InversearchError, Result};
use crate::r#type::DocId;
use crate::Index;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tokio::fs as tokio_fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 索引变更类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexChange {
    /// 添加文档
    Add {
        doc_id: DocId,
        content: String,
    },
    /// 删除文档
    Remove { doc_id: DocId },
    /// 更新文档
    Update {
        doc_id: DocId,
        content: String,
    },
}

/// WAL 配置
#[derive(Debug, Clone)]
pub struct WALConfig {
    /// 基础路径
    pub base_path: PathBuf,
    /// 最大 WAL 文件大小（字节）
    pub max_wal_size: usize,
    /// 是否启用压缩
    pub compression: bool,
    /// 压缩级别 (1-22)
    pub compression_level: i32,
}

impl Default for WALConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./wal"),
            max_wal_size: 100 * 1024 * 1024, // 100MB
            compression: true,
            compression_level: 3,
        }
    }
}

/// WAL 管理器
pub struct WALManager {
    config: WALConfig,
    wal_path: PathBuf,
    snapshot_path: PathBuf,
    wal_size: usize,
}

impl WALManager {
    /// 创建新的 WAL 管理器
    pub async fn new(config: WALConfig) -> Result<Self> {
        tokio_fs::create_dir_all(&config.base_path).await?;

        let wal_path = config.base_path.join("wal.log");
        let snapshot_path = config.base_path.join("snapshot.bin");

        // 获取当前 WAL 文件大小
        let wal_size = if wal_path.exists() {
            tokio_fs::metadata(&wal_path).await?.len() as usize
        } else {
            0
        };

        Ok(Self {
            config,
            wal_path,
            snapshot_path,
            wal_size,
        })
    }

    /// 记录变更到 WAL
    pub async fn record_change(&mut self, change: IndexChange) -> Result<()> {
        let serialized = bincode::serialize(&change)?;
        let encoded = base64::encode(&serialized);
        let line = format!("{}\n", encoded);

        let mut file = tokio_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .await?;

        file.write_all(line.as_bytes()).await?;
        file.sync_data().await?;

        self.wal_size += line.len();

        // WAL 超过阈值时触发快照
        if self.wal_size > self.config.max_wal_size {
            self.trigger_snapshot().await?;
        }

        Ok(())
    }

    /// 批量记录变更
    pub async fn record_changes(&mut self, changes: Vec<IndexChange>) -> Result<()> {
        if changes.is_empty() {
            return Ok(());
        }

        let mut lines = Vec::new();
        for change in changes {
            let serialized = bincode::serialize(&change)?;
            let encoded = base64::encode(&serialized);
            lines.push(format!("{}\n", encoded));
        }

        let mut file = tokio_fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.wal_path)
            .await?;

        for line in &lines {
            file.write_all(line.as_bytes()).await?;
            self.wal_size += line.len();
        }
        file.sync_data().await?;

        // WAL 超过阈值时触发快照
        if self.wal_size > self.config.max_wal_size {
            self.trigger_snapshot().await?;
        }

        Ok(())
    }

    /// 触发快照（异步任务）
    async fn trigger_snapshot(&mut self) -> Result<()> {
        // 注意：实际实现中应该使用后台任务
        // 这里简化处理，直接返回
        Ok(())
    }

    /// 创建快照
    pub async fn create_snapshot(&self, index: &Index) -> Result<()> {
        let temp_snapshot = self.config.base_path.join("snapshot.tmp");

        // 1. 序列化索引
        let snapshot_data = self.serialize_index(index)?;

        // 2. 压缩（如果启用）
        let final_data = if self.config.compression {
            compress_data(&snapshot_data, self.config.compression_level)?
        } else {
            snapshot_data
        };

        // 3. 写入临时文件
        let mut file = tokio_fs::File::create(&temp_snapshot).await?;
        file.write_all(&final_data).await?;
        file.sync_all().await?;
        drop(file);

        // 4. 原子替换
        tokio_fs::rename(&temp_snapshot, &self.snapshot_path).await?;

        // 5. 清空 WAL
        let _ = tokio_fs::remove_file(&self.wal_path).await;

        Ok(())
    }

    /// 加载索引（从快照 + WAL）
    pub async fn load(&self, index: &mut Index) -> Result<()> {
        // 1. 加载快照
        if self.snapshot_path.exists() {
            let mut file = tokio_fs::File::open(&self.snapshot_path).await?;
            let mut data = Vec::new();
            file.read_to_end(&mut data).await?;

            // 解压缩（如果启用）
            let snapshot_data = if self.config.compression {
                decompress_data(&data)?
            } else {
                data
            };

            self.deserialize_index(index, &snapshot_data)?;
        }

        // 2. 重放 WAL
        if self.wal_path.exists() {
            let file = tokio_fs::File::open(&self.wal_path).await?;
            let reader = BufReader::new(file.into_std().await);

            for line in reader.lines() {
                if let Ok(encoded) = line {
                    if let Ok(decoded) = base64::decode(&encoded) {
                        if let Ok(change) = bincode::deserialize::<IndexChange>(&decoded) {
                            self.apply_change(index, change)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// 应用变更到索引
    fn apply_change(&self, index: &mut Index, change: IndexChange) -> Result<()> {
        match change {
            IndexChange::Add { doc_id, content } => {
                index.add(doc_id, &content, false)?;
            }
            IndexChange::Remove { doc_id } => {
                index.remove(doc_id, false)?;
            }
            IndexChange::Update { doc_id, content } => {
                // 先移除再添加
                let _ = index.remove(doc_id, false);
                index.add(doc_id, &content, false)?;
            }
        }
        Ok(())
    }

    /// 序列化索引
    fn serialize_index(&self, index: &Index) -> Result<Vec<u8>> {
        use crate::serialize::SerializeConfig;
        let config = SerializeConfig::default();
        let export_data = index.export(&config)?;
        Ok(bincode::serialize(&export_data)?)
    }

    /// 反序列化索引
    fn deserialize_index(&self, index: &mut Index, data: &[u8]) -> Result<()> {
        use crate::serialize::{IndexExportData, SerializeConfig};
        let export_data: IndexExportData = bincode::deserialize(data)?;
        let config = SerializeConfig::default();
        index.import(export_data, &config)?;
        Ok(())
    }

    /// 清空 WAL 和快照
    pub async fn clear(&self) -> Result<()> {
        let _ = tokio_fs::remove_file(&self.wal_path).await;
        let _ = tokio_fs::remove_file(&self.snapshot_path).await;
        Ok(())
    }

    /// 获取 WAL 大小
    pub fn wal_size(&self) -> usize {
        self.wal_size
    }

    /// 获取快照大小
    pub async fn snapshot_size(&self) -> Result<u64> {
        if self.snapshot_path.exists() {
            Ok(tokio_fs::metadata(&self.snapshot_path).await?.len())
        } else {
            Ok(0)
        }
    }
}

/// 压缩数据
fn compress_data(data: &[u8], level: i32) -> Result<Vec<u8>> {
    zstd::stream::encode_all(data, level)
        .map_err(|e| InversearchError::Serialization(format!("Compression error: {}", e)))
}

/// 解压缩数据
fn decompress_data(data: &[u8]) -> Result<Vec<u8>> {
    zstd::stream::decode_all(data)
        .map_err(|e| InversearchError::Serialization(format!("Decompression error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_wal_basic() {
        let temp_dir = TempDir::new().unwrap();
        let config = WALConfig {
            base_path: temp_dir.path().to_path_buf(),
            max_wal_size: 1024 * 1024, // 1MB
            compression: false,
            compression_level: 3,
        };

        let mut wal = WALManager::new(config).await.unwrap();

        // 记录变更
        wal.record_change(IndexChange::Add {
            doc_id: 1,
            content: "hello world".to_string(),
        }).await.unwrap();

        wal.record_change(IndexChange::Add {
            doc_id: 2,
            content: "rust programming".to_string(),
        }).await.unwrap();

        // 验证 WAL 大小
        assert!(wal.wal_size() > 0);
    }

    #[tokio::test]
    async fn test_wal_with_compression() {
        let temp_dir = TempDir::new().unwrap();
        let config = WALConfig {
            base_path: temp_dir.path().to_path_buf(),
            max_wal_size: 1024 * 1024,
            compression: true,
            compression_level: 3,
        };

        let mut wal = WALManager::new(config).await.unwrap();

        // 记录变更
        wal.record_change(IndexChange::Add {
            doc_id: 1,
            content: "hello world".to_string(),
        }).await.unwrap();

        // 创建快照
        let mut index = Index::default();
        wal.create_snapshot(&index).await.unwrap();

        // 验证快照存在
        assert!(wal.snapshot_size().await.unwrap() > 0);
    }
}
