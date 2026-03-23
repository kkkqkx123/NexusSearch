//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use std::path::PathBuf;
use async_trait::async_trait;

pub mod redis;
pub mod wal;

/// 存储接口 - 类似JavaScript版本的StorageInterface
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    /// 挂载索引到存储
    async fn mount(&mut self, index: &Index) -> Result<()>;
    
    /// 打开连接
    async fn open(&mut self) -> Result<()>;
    
    /// 关闭连接
    async fn close(&mut self) -> Result<()>;
    
    /// 销毁数据库
    async fn destroy(&mut self) -> Result<()>;
    
    /// 提交索引变更
    async fn commit(&mut self, index: &Index, replace: bool, append: bool) -> Result<()>;
    
    /// 获取术语结果
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, resolve: bool, enrich: bool) -> Result<SearchResults>;
    
    /// 富化结果
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    
    /// 检查ID是否存在
    async fn has(&self, id: DocId) -> Result<bool>;
    
    /// 删除ID
    async fn remove(&mut self, ids: &[DocId]) -> Result<()>;
    
    /// 清空数据
    async fn clear(&mut self) -> Result<()>;
    
    /// 获取存储信息
    async fn info(&self) -> Result<StorageInfo>;
}

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub index_count: usize,
    pub is_connected: bool,
}

/// 内存存储实现 - 用于测试和开发
pub struct MemoryStorage {
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    is_open: bool,
    memory_usage: AtomicUsize,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
}

impl MemoryStorage {
    /// 创建新的内存存储
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
            is_open: false,
            memory_usage: AtomicUsize::new(0),
            operation_count: AtomicUsize::new(0),
            total_latency: AtomicUsize::new(0),
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> MemoryStorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed);
        let total_latency = self.total_latency.load(Ordering::Relaxed);
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        MemoryStorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            error_count: 0,
        }
    }

    /// 更新内存使用量
    fn update_memory_usage(&self) {
        let mut total_size = 0;
        
        // 计算数据大小
        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }
        
        // 计算上下文数据大小
        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }
        
        // 计算文档大小
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

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存存储性能指标
#[derive(Debug, Clone)]
pub struct MemoryStorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub error_count: usize,
}

#[async_trait::async_trait]
impl StorageInterface for MemoryStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }
    
    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        Ok(())
    }
    
    async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        Ok(())
    }
    
    async fn destroy(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.is_open = false;
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();
        
        // 从索引导出数据到存储
        for (_term_hash, doc_ids) in &index.map.index {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }
        
        // 导出上下文数据
        for (_ctx_key, ctx_map) in &index.ctx.index {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_insert_with(HashMap::new)
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }
        
        self.update_memory_usage();
        self.record_operation_completion(start_time);
        
        Ok(())
    }
    
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        let results = if let Some(ctx_key) = ctx {
            // 上下文搜索
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
            // 普通搜索
            if let Some(doc_ids) = self.data.get(key) {
                apply_limit_offset(doc_ids, limit, offset)
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
        for (_term, doc_ids) in &self.data {
            if doc_ids.contains(&id) {
                return Ok(true);
            }
        }
        
        // 检查上下文数据
        for (_ctx, ctx_map) in &self.context_data {
            for (_term, doc_ids) in ctx_map {
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
        Ok(())
    }
    
    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        Ok(())
    }
    
    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "MemoryStorage".to_string(),
            version: "0.1.0".to_string(),
            size: (self.data.len() + self.context_data.len() + self.documents.len()) as u64,
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: self.is_open,
        })
    }
}

/// 文件存储实现 - 简单的文件持久化
pub struct FileStorage {
    file_path: String,
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
}

impl FileStorage {
    /// 创建新的文件存储
    pub fn new(file_path: impl Into<String>) -> Self {
        Self {
            file_path: file_path.into(),
            data: HashMap::new(),
            context_data: HashMap::new(),
            documents: HashMap::new(),
        }
    }
    
    /// 保存到文件
    pub async fn save_to_file(&self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncWriteExt;
        
        let data = serde_json::json!({
            "data": self.data,
            "context_data": self.context_data,
            "documents": self.documents,
        });
        
        let json_str = serde_json::to_string_pretty(&data)?;
        let mut file = File::create(&self.file_path).await?;
        file.write_all(json_str.as_bytes()).await?;
        
        Ok(())
    }
    
    /// 从文件加载
    pub async fn load_from_file(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let mut file = match File::open(&self.file_path).await {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        if contents.trim().is_empty() {
            return Ok(());
        }

        let data: serde_json::Value = serde_json::from_str(&contents)?;

        self.data = serde_json::from_value(data["data"].clone())?;
        self.context_data = serde_json::from_value(data["context_data"].clone())?;
        self.documents = serde_json::from_value(data["documents"].clone())?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        // 尝试从文件加载现有数据
        if let Err(_) = self.load_from_file().await {
            // 文件不存在或加载失败，创建新的空存储
        }
        Ok(())
    }
    
    async fn open(&mut self) -> Result<()> {
        self.load_from_file().await
    }
    
    async fn close(&mut self) -> Result<()> {
        self.save_to_file().await
    }
    
    async fn destroy(&mut self) -> Result<()> {
        use tokio::fs;
        
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        
        // 删除文件
        if let Err(_) = fs::remove_file(&self.file_path).await {
            // 文件可能不存在，忽略错误
        }
        
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 从索引导出数据
        for (_term_hash, doc_ids) in &index.map.index {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }
        
        // 导出上下文数据
        for (_ctx_key, ctx_map) in &index.ctx.index {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_insert_with(HashMap::new)
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }
        
        // 保存到文件
        self.save_to_file().await
    }
    
    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
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
        Ok(self.documents.contains_key(&id))
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
        Ok(())
    }
    
    async fn clear(&mut self) -> Result<()> {
        self.data.clear();
        self.context_data.clear();
        self.documents.clear();
        self.save_to_file().await
    }
    
    async fn info(&self) -> Result<StorageInfo> {
        let file_size = if let Ok(metadata) = std::fs::metadata(&self.file_path) {
            metadata.len()
        } else {
            0
        };
        
        Ok(StorageInfo {
            name: "FileStorage".to_string(),
            version: "0.1.0".to_string(),
            size: file_size,
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: true,
        })
    }
}

/// 应用限制和偏移的辅助函数
fn apply_limit_offset(results: &[DocId], limit: usize, offset: usize) -> SearchResults {
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

/// 高级文件系统存储实现 - 支持内存监控和性能指标
pub struct AdvancedFileStorage {
    base_path: PathBuf,
    data: HashMap<String, Vec<DocId>>,
    context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    documents: HashMap<DocId, String>,
    memory_usage: AtomicUsize,
    operation_count: AtomicUsize,
    total_latency: AtomicUsize,
    is_open: bool,
}

impl AdvancedFileStorage {
    /// 创建新的高级文件存储
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

    /// 保存到文件（使用更高效的序列化）
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
        
        // 使用 MessagePack 格式进行序列化（更高效）
        let serialized = rmp_serde::to_vec(&data)
            .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
        
        let data_file = self.base_path.join("data.msgpack");
        let mut file = File::create(&data_file).await?;
        file.write_all(&serialized).await?;
        
        Ok(())
    }
    
    /// 从文件加载
    pub async fn load_from_file(&mut self) -> Result<()> {
        use tokio::fs::File;
        use tokio::io::AsyncReadExt;

        let data_file = self.base_path.join("data.msgpack");
        
        let mut file = match File::open(&data_file).await {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;

        if contents.is_empty() {
            return Ok(());
        }

        let data: FileStorageData = rmp_serde::from_slice(&contents)
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        self.data = data.data;
        self.context_data = data.context_data;
        self.documents = data.documents;
        
        // 更新内存使用量
        self.update_memory_usage();

        Ok(())
    }
    
    /// 更新内存使用量
    fn update_memory_usage(&self) {
        let mut total_size = 0;
        
        // 计算数据大小
        total_size += std::mem::size_of_val(&self.data);
        for (k, v) in &self.data {
            total_size += k.len() + v.len() * std::mem::size_of::<DocId>();
        }
        
        // 计算上下文数据大小
        total_size += std::mem::size_of_val(&self.context_data);
        for (ctx_key, ctx_map) in &self.context_data {
            total_size += ctx_key.len();
            total_size += std::mem::size_of_val(ctx_map);
            for (term, ids) in ctx_map {
                total_size += term.len() + ids.len() * std::mem::size_of::<DocId>();
            }
        }
        
        // 计算文档大小
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

/// 文件存储数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageData {
    pub version: String,
    pub timestamp: String,
    pub data: HashMap<String, Vec<DocId>>,
    pub context_data: HashMap<String, HashMap<String, Vec<DocId>>>,
    pub documents: HashMap<DocId, String>,
}

/// 文件存储性能指标
#[derive(Debug, Clone)]
pub struct FileStorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub file_size: u64,
    pub error_count: usize,
}

#[async_trait::async_trait]
impl StorageInterface for AdvancedFileStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        // 确保基础目录存在
        tokio::fs::create_dir_all(&self.base_path).await?;
        
        // 尝试从文件加载现有数据
        if let Err(e) = self.load_from_file().await {
            eprintln!("Failed to load from file: {}", e);
            // 文件不存在或加载失败，创建新的空存储
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
        
        // 删除文件
        let data_file = self.base_path.join("data.msgpack");
        if let Err(_) = tokio::fs::remove_file(&data_file).await {
            // 文件可能不存在，忽略错误
        }
        
        self.update_memory_usage();
        self.is_open = false;
        
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();
        
        // 从索引导出数据
        for (_term_hash, doc_ids) in &index.map.index {
            for (term_str, ids) in doc_ids {
                self.data.insert(term_str.clone(), ids.clone());
            }
        }
        
        // 导出上下文数据
        for (_ctx_key, ctx_map) in &index.ctx.index {
            for (ctx_term, doc_ids) in ctx_map {
                self.context_data.entry("default".to_string())
                    .or_insert_with(HashMap::new)
                    .insert(ctx_term.clone(), doc_ids.clone());
            }
        }
        
        // 保存到文件
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
            name: "AdvancedFileStorage".to_string(),
            version: "1.0.0".to_string(),
            size: self.get_file_size(),
            document_count: self.documents.len(),
            index_count: self.data.len(),
            is_connected: self.is_open,
        })
    }
}

/// WAL 存储实现 - 支持增量持久化
pub struct WALStorage {
    wal_manager: wal::WALManager,
    documents: HashMap<DocId, String>,
    is_open: bool,
}

impl WALStorage {
    /// 创建新的 WAL 存储
    pub async fn new(config: wal::WALConfig) -> Result<Self> {
        let wal_manager = wal::WALManager::new(config).await?;

        Ok(Self {
            wal_manager,
            documents: HashMap::new(),
            is_open: false,
        })
    }

    /// 创建快照
    pub async fn create_snapshot(&self, index: &Index) -> Result<()> {
        self.wal_manager.create_snapshot(index).await
    }
}

#[async_trait::async_trait]
impl StorageInterface for WALStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        self.is_open = true;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.is_open = false;
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        self.documents.clear();
        self.wal_manager.clear().await?;
        self.is_open = false;
        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        // 使用 WAL 创建快照
        self.wal_manager.create_snapshot(index).await
    }

    async fn get(&self, key: &str, ctx: Option<&str>, limit: usize, offset: usize, _resolve: bool, _enrich: bool) -> Result<SearchResults> {
        // WAL 存储需要通过加载索引来获取数据
        // 这里简化处理，返回空结果
        // 实际应用中应该维护一个内存索引
        let _ = (key, ctx, limit, offset);
        Ok(Vec::new())
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
        Ok(self.documents.contains_key(&id))
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        for &id in ids {
            self.documents.remove(&id);
            self.wal_manager.record_change(wal::IndexChange::Remove { doc_id: id }).await?;
        }
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.documents.clear();
        self.wal_manager.clear().await?;
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let wal_size = self.wal_manager.wal_size() as u64;
        let snapshot_size = self.wal_manager.snapshot_size().await?;

        Ok(StorageInfo {
            name: "WALStorage".to_string(),
            version: "0.1.0".to_string(),
            size: wal_size + snapshot_size,
            document_count: self.documents.len(),
            index_count: 0,
            is_connected: self.is_open,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Index;

    #[tokio::test]
    async fn test_memory_storage() {
        let mut storage = MemoryStorage::new();
        storage.open().await.unwrap();

        let mut index = Index::default();
        index.add(1, "hello world", false).unwrap();
        index.add(2, "rust programming", false).unwrap();

        // 提交到存储
        storage.commit(&index, false, false).await.unwrap();

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.unwrap();
        println!("Get results: {:?}", results);
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 测试存在检查
        println!("Checking has(1)");
        let has_result = storage.has(1).await.unwrap();
        println!("has(1) result: {}", has_result);
        assert!(has_result);
        assert!(!storage.has(3).await.unwrap());

        // 测试删除
        storage.remove(&[1]).await.unwrap();
        assert!(!storage.has(1).await.unwrap());
        
        storage.close().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_file_storage() {
        use tempfile::NamedTempFile;
        
        let temp_file = NamedTempFile::new().unwrap();
        let file_path = temp_file.path().to_str().unwrap().to_string();
        
        let mut storage = FileStorage::new(file_path);
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
        let mut storage2 = FileStorage::new(temp_file.path().to_str().unwrap().to_string());
        storage2.open().await.unwrap();
        
        let results2 = storage2.get("test", None, 10, 0, true, false).await.unwrap();
        assert_eq!(results2.len(), 2);
        
        storage2.destroy().await.unwrap();
    }
}