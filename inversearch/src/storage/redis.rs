use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::r#trait::StorageInterface;
use crate::storage::common::types::StorageInfo;
use crate::Index;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct RedisStorageConfig {
    pub url: String,
    pub pool_size: usize,
    pub connection_timeout: Duration,
    pub key_prefix: String,
}

impl Default for RedisStorageConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            pool_size: 10,
            connection_timeout: Duration::from_secs(5),
            key_prefix: "inversearch".to_string(),
        }
    }
}

pub struct RedisStorage {
    client: RedisClient,
    config: RedisStorageConfig,
    key_prefix: String,
    connection_pool: Arc<RwLock<MultiplexedConnection>>,
    memory_usage: Arc<AtomicUsize>,
    operation_count: Arc<AtomicU64>,
    total_latency: Arc<AtomicU64>,
    error_count: Arc<AtomicU64>,
    last_operation_time: Arc<std::sync::Mutex<Option<Instant>>>,
}

impl RedisStorage {
    pub async fn new(config: RedisStorageConfig) -> Result<Self> {
        let key_prefix = config.key_prefix.clone();
        let client = RedisClient::open(config.url.as_str())
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let _: String = redis::cmd("PING")
            .query_async(&mut conn.clone())
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(Self {
            client,
            config,
            key_prefix,
            connection_pool: Arc::new(RwLock::new(conn)),
            memory_usage: Arc::new(AtomicUsize::new(0)),
            operation_count: Arc::new(AtomicU64::new(0)),
            total_latency: Arc::new(AtomicU64::new(0)),
            error_count: Arc::new(AtomicU64::new(0)),
            last_operation_time: Arc::new(std::sync::Mutex::new(None)),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    fn make_index_key(&self, term: &str) -> String {
        self.make_key(&format!("index:{}", term))
    }

    fn make_context_key(&self, context: &str, term: &str) -> String {
        self.make_key(&format!("ctx:{}:{}", context, term))
    }

    fn make_doc_key(&self, doc_id: DocId) -> String {
        self.make_key(&format!("doc:{}", doc_id))
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()).into())
    }

    async fn with_connection<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&mut MultiplexedConnection) -> redis::RedisFuture<'_, R>,
    {
        let mut conn_guard = self.connection_pool.write().await;
        let result = f(&mut conn_guard).await.map_err(|e| {
            self.error_count.fetch_add(1, Ordering::Relaxed);
            crate::error::StorageError::Connection(e.to_string())
        })?;
        Ok(result)
    }

    async fn scan_keys(&self, pattern: &str) -> Result<Vec<String>> {
        let mut conn = self.get_connection().await?;
        let mut cursor = 0u64;
        let mut all_keys = Vec::new();

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

            all_keys.extend(keys);
            cursor = next_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
    }

    /// 批量提交（使用 MSET）
    pub async fn commit_batch(&mut self, index: &Index, batch_size: usize) -> Result<()> {
        let mut conn = self.get_connection().await?;

        // 收集所有索引项
        let mut index_items: Vec<(String, String)> = Vec::new();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                let key = self.make_index_key(term_str);
                let serialized = serde_json::to_string(ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                let key = self.make_context_key("default", ctx_term);
                let serialized = serde_json::to_string(doc_ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        // 使用 MSET 批量设置
        for chunk in index_items.chunks(batch_size) {
            let mut mset_args: Vec<(&str, &str)> = Vec::new();
            for (key, value) in chunk {
                mset_args.push((key.as_str(), value.as_str()));
            }

            let _: () = redis::cmd("MSET")
                .arg(mset_args.as_slice())
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl StorageInterface for RedisStorage {
    async fn mount(&mut self, _index: &Index) -> Result<()> {
        Ok(())
    }

    async fn open(&mut self) -> Result<()> {
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        if !keys.is_empty() {
            let mut conn = self.get_connection().await?;
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        Ok(())
    }

    async fn commit(&mut self, index: &Index, _replace: bool, _append: bool) -> Result<()> {
        let start_time = self.record_operation_start();

        let mut conn = self.get_connection().await?;

        // 收集所有索引项
        let mut index_items: Vec<(String, String)> = Vec::new();

        for doc_ids in index.map.index.values() {
            for (term_str, ids) in doc_ids {
                let key = self.make_index_key(term_str);
                let serialized = serde_json::to_string(ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        for ctx_map in index.ctx.index.values() {
            for (ctx_term, doc_ids) in ctx_map {
                let key = self.make_context_key("default", ctx_term);
                let serialized = serde_json::to_string(doc_ids)
                    .map_err(|e| crate::error::StorageError::Serialization(e.to_string()))?;
                index_items.push((key, serialized));
            }
        }

        // 使用 Pipeline 批量提交（每批 1000 个）
        const BATCH_SIZE: usize = 1000;

        for chunk in index_items.chunks(BATCH_SIZE) {
            let mut batch_pipe = redis::pipe();
            for (key, value) in chunk {
                batch_pipe.cmd("SET").arg(key).arg(value);
            }
            let _: () = batch_pipe
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
        }

        // 更新内存使用量
        self.update_memory_usage().await?;
        self.record_operation_completion(start_time);

        Ok(())
    }

    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        limit: usize,
        offset: usize,
        _resolve: bool,
        _enrich: bool,
    ) -> Result<SearchResults> {
        let mut conn = self.get_connection().await?;

        let redis_key = if let Some(ctx_key) = ctx {
            self.make_context_key(ctx_key, key)
        } else {
            self.make_index_key(key)
        };

        let serialized: String = redis::cmd("GET")
            .arg(&redis_key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        if serialized.is_empty() {
            return Ok(Vec::new());
        }

        let doc_ids: Vec<DocId> = serde_json::from_str(&serialized)
            .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;

        let start = offset.min(doc_ids.len());
        let end = if limit > 0 {
            (start + limit).min(doc_ids.len())
        } else {
            doc_ids.len()
        };

        Ok(doc_ids[start..end].to_vec())
    }

    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let keys: Vec<String> = ids.iter().map(|&id| self.make_doc_key(id)).collect();
        let mut conn = self.get_connection().await?;

        let serialized_list: Vec<String> = redis::cmd("MGET")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        let mut results = Vec::new();
        for (i, serialized) in serialized_list.into_iter().enumerate() {
            if !serialized.is_empty() {
                let doc: serde_json::Value = serde_json::from_str(&serialized)
                    .map_err(|e| crate::error::StorageError::Deserialization(e.to_string()))?;
                results.push(crate::r#type::EnrichedSearchResult {
                    id: ids[i],
                    doc: Some(doc),
                    highlight: None,
                });
            }
        }

        Ok(results)
    }

    async fn has(&self, id: DocId) -> Result<bool> {
        let mut conn = self.get_connection().await?;
        let key = self.make_doc_key(id);

        let exists: bool = redis::cmd("EXISTS")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(exists)
    }

    async fn remove(&mut self, ids: &[DocId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let keys: Vec<String> = ids.iter().map(|&id| self.make_doc_key(id)).collect();
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        self.destroy().await
    }

    async fn info(&self) -> Result<StorageInfo> {
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        let doc_pattern = format!("{}:doc:*", self.key_prefix);
        let doc_keys = self.scan_keys(&doc_pattern).await?;

        let index_pattern = format!("{}:index:*", self.key_prefix);
        let index_keys = self.scan_keys(&index_pattern).await?;

        let mut total_size = 0u64;
        for key in &keys {
            let mut conn = self.get_connection().await?;
            let size: Option<usize> = redis::cmd("STRLEN")
                .arg(key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            total_size += size.unwrap_or(0) as u64;
        }

        Ok(StorageInfo {
            name: "RedisStorage".to_string(),
            version: "0.1.0".to_string(),
            size: total_size,
            document_count: doc_keys.len(),
            index_count: index_keys.len(),
            is_connected: true,
        })
    }
}

impl RedisStorage {
    /// 批量删除文档（优化版本，使用 pipeline）
    pub async fn remove_batch(&mut self, ids: &[DocId]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let keys: Vec<String> = ids.iter().map(|&id| self.make_doc_key(id)).collect();
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("DEL")
            .arg(&keys)
            .query_async(&mut conn)
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;

        Ok(())
    }

    /// 连接池管理
    pub async fn get_pooled_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| crate::error::StorageError::Connection(e.to_string()).into())
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        match self.get_connection().await {
            Ok(mut conn) => {
                let result: String = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
                Ok(result == "PONG")
            }
            Err(_) => Ok(false),
        }
    }

    /// 获取内存使用情况
    pub fn get_memory_usage(&self) -> usize {
        self.memory_usage.load(Ordering::Relaxed)
    }

    /// 获取操作统计
    pub fn get_operation_stats(&self) -> StorageMetrics {
        let operation_count = self.operation_count.load(Ordering::Relaxed) as usize;
        let total_latency = self.total_latency.load(Ordering::Relaxed) as usize;
        let avg_latency = if operation_count > 0 {
            total_latency / operation_count
        } else {
            0
        };

        StorageMetrics {
            operation_count,
            average_latency: avg_latency,
            memory_usage: self.get_memory_usage(),
            error_count: self.error_count.load(Ordering::Relaxed) as usize,
        }
    }

    /// 记录操作开始时间（内部使用）
    fn record_operation_start(&self) -> Instant {
        let start_time = Instant::now();
        if let Ok(mut last_op) = self.last_operation_time.lock() {
            *last_op = Some(start_time);
        }
        start_time
    }

    /// 记录操作完成（内部使用）
    fn record_operation_completion(&self, start_time: Instant) {
        let latency = start_time.elapsed().as_micros() as u64;
        self.operation_count.fetch_add(1, Ordering::Relaxed);
        self.total_latency.fetch_add(latency, Ordering::Relaxed);
    }

    /// 更新内存使用量估计
    async fn update_memory_usage(&self) -> Result<()> {
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        let mut total_size = 0usize;
        for key in &keys {
            let mut conn = self.get_connection().await?;
            let size: Option<usize> = redis::cmd("STRLEN")
                .arg(key)
                .query_async(&mut conn)
                .await
                .map_err(|e| crate::error::StorageError::Connection(e.to_string()))?;
            total_size += size.unwrap_or(0);
        }

        self.memory_usage.store(total_size, Ordering::Relaxed);
        Ok(())
    }
}

/// 存储性能指标
#[derive(Debug, Clone, Default)]
pub struct StorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize, // 微秒
    pub memory_usage: usize,
    pub error_count: usize,
}

impl StorageMetrics {
    /// 创建空的指标
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置所有指标
    pub fn reset(&mut self) {
        self.operation_count = 0;
        self.average_latency = 0;
        self.memory_usage = 0;
        self.error_count = 0;
    }
}
