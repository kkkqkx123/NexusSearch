//! 内存存储实现
//!
//! 提供基于内存的存储后端，数据不持久化

use crate::r#type::{SearchResults, EnrichedSearchResults, DocId};
use crate::error::Result;
use crate::Index;
use crate::storage::interface::StorageInterface;
use crate::storage::types::{StorageInfo, MemoryStorageMetrics};
use crate::storage::utils::apply_limit_offset;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// 内存存储
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
}
