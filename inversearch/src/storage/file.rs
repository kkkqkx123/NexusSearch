//! 文件存储实现
//!
//! 提供基于文件的持久化存储后端

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::interface::StorageInterface;
use crate::storage::types::{StorageInfo, FileStorageData, FileStorageMetrics};
use crate::storage::utils::apply_limit_offset;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// 文件存储
pub struct FileStorage {
    base_path: PathBuf,
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    memory_usage: AtomicUsize,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
    is_open: bool,
}

impl FileStorage {
    /// 创建新的文件存储
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
            is_open: false,
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> FileStorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        FileStorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            file_size: self.get_file_size(),
            error_count: 0,
        }
    }

    /// 获取文件大小
    pub fn get_file_size(&self) -> u64 {
        let data_file = self.base_path.join("data.msgpack");
        if let Ok(metadata) = std::fs::metadata(data_file) {
            metadata.len()
        } else {
            0
        }
    }

    /// 保存到文件
    pub async fn save_to_file(&self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;

        let data = FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: self.data.clone(),
            context_data: self.context_data.clone(),
            documents: self.documents.clone(),
        };

        // 使用 bincode 进行序列化（高效）
        let serialized = bincode::serialize(&data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

        let data_file = self.base_path.join("data.bin");
        let mut file = File::create(&data_file).await?;
        file.write_all(&serialized).await?;

        Ok(())
    }

    /// 从文件加载
    pub async fn load_from_file(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let data_file = self.base_path.join("data.bin");

        let mut file = match File::open(&data_file).await {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        if contents.is_empty() {
            return Ok(());
        }

        let data: FileStorageData = bincode::deserialize(&contents)
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        self.data = data.data;
        self.context_data = data.context_data;
        self.documents = data.documents;

        self.update_memory_usage();

        Ok(())
    }

    /// 更新内存使用量
    fn update_memory_usage(&self) {
        let mut total_size = 0;

        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }

        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }

        total_size += std::mem::size_of_val(&self.documents);
        for (id, content) in &self.documents {
            total_size += std::mem::size_of_val(id) + content.len();
        }

        self.memory_usage.store(total_size, Ordering::Relaxed);
    }

    /// 记录操作开始时间
    fn record_operation_start(&self) -> Instant {
        Instant::now()
    }

    /// 记录操作完成
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as usize;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        tokio::fs::create_dir_all(&self.base_path).await?;

        if let Err(e) = self.load_from_file().await {
            eprintln!("Failed to load from file: {}", e);
        }
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        self.load_from_file().await
    }

    async fn close(&mut self) -> Result<()> {
        self.save_to_file().await?;
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();

        let data_file = self.base_path.join("data.bin");
        let _ = tokio::fs::remove_file(&data_file).await;

        self.update_memory_usage();
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }

        self.save_to_file().await?;
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        Ok(())
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let start_time = self.record_operation_start();

        let results = if let Some(ctx_key) = ctx {
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    apply_limit_offset(doc_ids, limit, offset)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            if let Some(doc_ids) = self.data.get(key) {
                apply_limit_offset(doc_ids, limit, offset)
            } else {
                Vec::new()
            }
        };

        self.record_operation_completion(start_time);
        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        let start_time = self.record_operation_start();
        let mut results = Vec::new();

        for &id in ids {
            if let Some(content) = self.documents.get(&id) {
                results.push(crate::r#type::EnrichedSearchResult {
                    id,
                    doc: Some(serde_json::json!({
                        "content": content,
                        "id": id
                    })),
                    highlight: None,
                });
            }
        }

        self.record_operation_completion(start_time);
        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let start_time = self.record_operation_start();
        let result = Ok(self.documents.contains_key(&id));
        self.record_operation_completion(start_time);
        result
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        let start_time = self.record_operation_start();

        for &id in ids {
            self.documents.remove(&id);

            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }

            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }

        self.save_to_file().await?;
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        let start_time = self.record_operation_start();

        self.data.clear();
        self.context_data.clear();
        self.documents.clear();

        self.save_to_file().await?;
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "1.0.0".to_string(),
            size: self.get_file_size(),
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: self.is_open,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_file_storage() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        let dir_path = temp_dir.path();
        
        let mut storage = FileStorage::new(dir_path.to_str().unwrap().to_string());
        storage.open().await.unwrap();
        
        let mut index = Index::default();
        index.add(1, "test document", false).unwrap();
        index.add(2, "another test", false).unwrap();
        
        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();
        
        // 测试获取
        let results = storage.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results.contains(&1));
        assert!(results.contains(&2));
        
        // 关闭存储（会保存到文件）
        storage.close().await.unwrap();
        
        // 重新打开并验证数据还在
        let mut storage2 = FileStorage::new(dir_path.to_str().unwrap().to_string());
        storage2.open().await.unwrap();
        
        let results2 = storage2.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results2.len(), 2);
        
        storage2.destroy().await.unwrap();
    }
}
