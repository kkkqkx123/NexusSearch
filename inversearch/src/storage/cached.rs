//! 缓存存储实现
//!
//! 提供内存缓存 + 持久化存储的组合实现
//! 作为默认存储后端，兼顾性能和数据安全

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::interface::StorageInterface;
use crate::storage::types::StorageInfo;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// 缓存存储配置
#[derive(Debug, Clone)]
pub struct CachedStorageConfig {
    /// 基础路径
    pub base_path: PathBuf,
    /// 自动保存间隔（秒），0 表示不自动保存
    pub auto_save_interval: u64,
    /// 是否在 drop 时自动保存
    pub auto_save_on_drop: bool,
}

impl Default for CachedStorageConfig {
    fn default() -> Self {
        Self {
            base_path: PathBuf::from("./data"),
            auto_save_interval: 0,  // 默认不自动保存，由用户控制
            auto_save_on_drop: true,
        }
    }
}

/// 缓存存储
/// 
/// 结合内存存储的性能和文件存储的持久化能力
/// - 所有读写操作先在内存中进行
/// - 显式调用 `save()` 或 `close()` 时持久化到文件
/// - 打开时自动从文件加载数据
pub struct CachedStorage {
    config: CachedStorageConfig,
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    memory_usage: AtomicUsize,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
    is_open: bool,
    is_dirty: bool,  // 标记是否有未保存的变更
}

impl CachedStorage {
    /// 使用默认配置创建缓存存储
    pub fn new() -> Self {
        Self::with_config(CachedStorageConfig::default())
    }

    /// 使用指定路径创建缓存存储
    pub fn with_path(base_path: impl Into<PathBuf>) -> Self {
        let mut config = CachedStorageConfig::default();
        config.base_path = base_path.into();
        Self::with_config(config)
    }

    /// 使用自定义配置创建缓存存储
    pub fn with_config(config: CachedStorageConfig) -> Self {
        Self {
            config,
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
            is_open: false,
            is_dirty: false,
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_count(&self) -> usize {
        self.operation_count.load(Ordering::Relaxed)
    }

    /// 检查是否有未保存的变更
    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// 获取配置
    pub fn config(&self) -> &CachedStorageConfig {
        &self.config
    }

    /// 保存到文件
    pub async fn save(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        use crate::storage::types::FileStorageData;

        let data = FileStorageData {
            version: "1.0.0".to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            data: self.data.clone(),
            context_data: self.context_data.clone(),
            documents: self.documents.clone(),
        };

        // 确保目录存在
        tokio::fs::create_dir_all(&self.config.base_path).await?;

        // 使用 bincode 进行序列化
        let serialized = bincode::serialize(&data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;

        // 写入临时文件，然后原子替换
        let data_file = self.config.base_path.join("data.bin");
        let temp_file = self.config.base_path.join("data.bin.tmp");
        
        let mut file = File::create(&temp_file).await?;
        file.write_all(&serialized).await?;
        file.sync_all().await?;
        drop(file);

        // 原子替换
        tokio::fs::rename(&temp_file, &data_file).await?;

        // 重置脏标记
        self.is_dirty = false;

        Ok(())
    }

    /// 从文件加载
    pub async fn load(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;
        use crate::storage::types::FileStorageData;

        let data_file = self.config.base_path.join("data.bin");

        let mut file = match File::open(&data_file).await {
            Ok(f) => f,
            Err(_) => return Ok(()),  // 文件不存在，视为空存储
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
        self.is_dirty = false;

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

    /// 应用限制和偏移的辅助函数
    fn apply_limit_offset(&self, results: &[DocId], limit: usize, offset: usize) -> SearchResults {
        if results.is_empty() {
            return Vec::new();
        }

        let start = offset.min(results.len());
        let end = if limit > 0 {
            (start + limit).min(results.len())
        } else {
            results.len()
        };

        results[start..end].to_vec()
    }
}

impl Default for CachedStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl StorageInterface for CachedStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        tokio::fs::create_dir_all(&self.config.base_path).await?;
        self.load().await
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        self.load().await
    }

    async fn close(&mut self) -> Result<()> {
        if self.is_dirty {
            self.save().await?;
        }
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.is_dirty = false;

        let data_file = self.config.base_path.join("data.bin");
        let _ = tokio::fs::remove_file(&data_file).await;

        self.update_memory_usage();
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        // 从索引导出数据到存储
        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }

        // 导出上下文数据
        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_default()
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }

        self.is_dirty = true;
        self.update_memory_usage();
        self.record_operation_completion(start_time);

        Ok(())
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let results = if let Some(ctx_key) = ctx {
            // 上下文搜索
            if let Some(ctx_map) = self.context_data.get(ctx_key) {
                if let Some(doc_ids) = ctx_map.get(key) {
                    self.apply_limit_offset(doc_ids, limit, offset)
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        } else {
            // 普通搜索
            if let Some(doc_ids) = self.data.get(key) {
                self.apply_limit_offset(doc_ids, limit, offset)
            } else {
                Vec::new()
            }
        };

        Ok(results)
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
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

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        // 检查文档ID是否存在于索引数据中
        for doc_ids in self.data.values() {
            if doc_ids.contains(&id) {
                return Ok(true);
            }
        }

        // 检查上下文数据
        for ctx_map in self.context_data.values() {
            for doc_ids in ctx_map.values() {
                if doc_ids.contains(&id) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);

            // 从索引数据中移除
            for doc_ids in self.data.values_mut() {
                doc_ids.retain(|&doc_id| doc_id != id);
            }

            // 从上下文数据中移除
            for ctx_map in self.context_data.values_mut() {
                for doc_ids in ctx_map.values_mut() {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
            }
        }

        self.is_dirty = true;
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.is_dirty = true;
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let file_size = if self.config.base_path.exists() {
            let data_file = self.config.base_path.join("data.bin");
            std::fs::metadata(&data_file).map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        Ok(StorageInfo {
            name: "CachedStorage".to_string(),
            version: "1.0.0".to_string(),
            size: file_size,
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
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_cached_storage_basic() {
        let temp_dir = TempDir::new().unwrap();
        let mut storage = CachedStorage::with_path(temp_dir.path());
        
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();

        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();
        assert!(storage.is_dirty());

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 关闭存储（会保存到文件）
        storage.close().await.unwrap();
        assert!(!storage.is_dirty());

        // 重新打开并验证数据还在
        let mut storage2 = CachedStorage::with_path(temp_dir.path());
        storage2.open().await.unwrap();
        
        let results2 = storage2.get("hello", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results2.len(), 1);
        
        storage2.destroy().await.unwrap();
    }

    #[tokio::test]
    async fn test_cached_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // 第一次创建并写入数据
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.unwrap();
            
            let mut index = Index::default();
            index.add(1, "persistent data", false).unwrap();
            storage.commit(&index, false, false).await.unwrap();
            
            storage.close().await.unwrap();
        }

        // 第二次打开验证数据持久化
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.unwrap();
            
            let results = storage.get("persistent", None, 10, 0, true, false).await.unwrap();
            assert_eq!(results.len(), 1);
            assert!(results.contains(&1));
            
            storage.destroy().await.unwrap();
        }
    }
}
