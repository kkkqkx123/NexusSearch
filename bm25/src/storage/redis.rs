//! Redis 存储实现
//!
//! 使用 Redis 存储 BM25 词频统计信息
//! 数据结构：
//! - BM25:tf:{term} -> Hash { doc_id: tf_value }
//! - BM25:df:{term} -> String (df_value)
//! - BM25:stats -> Hash { total_docs, avg_doc_length }

use crate::error::{Bm25Error, Result};
use crate::storage::common::trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use redis::{aio::MultiplexedConnection, Client as RedisClient};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Redis 存储配置
#[derive(Debug, Clone)]
pub struct RedisStorageConfig {
    pub url: String,
    pub connection_timeout: Duration,
    pub key_prefix: String,
}

impl Default for RedisStorageConfig {
    fn default() -> Self {
        Self {
            url: "redis://127.0.0.1:6379".to_string(),
            connection_timeout: Duration::from_secs(5),
            key_prefix: "bm25".to_string(),
        }
    }
}

/// Redis 存储实现
pub struct RedisStorage {
    client: RedisClient,
    config: RedisStorageConfig,
    key_prefix: String,
    connection_pool: Arc<RwLock<MultiplexedConnection>>,
}

impl RedisStorage {
    pub async fn new(config: RedisStorageConfig) -> Result<Self> {
        let key_prefix = config.key_prefix.clone();
        let client = RedisClient::open(config.url.as_str())
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        let _: String = redis::cmd("PING")
            .query_async(&conn)
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        Ok(Self {
            client,
            config,
            key_prefix,
            connection_pool: Arc::new(RwLock::new(conn)),
        })
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    fn make_tf_key(&self, term: &str) -> String {
        self.make_key(&format!("tf:{}", term))
    }

    fn make_df_key(&self, term: &str) -> String {
        self.make_key(&format!("df:{}", term))
    }

    async fn get_connection(&self) -> Result<MultiplexedConnection> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()).into())
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
                .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

            all_keys.extend(keys);
            cursor = next_cursor;

            if cursor == 0 {
                break;
            }
        }

        Ok(all_keys)
    }
}

#[async_trait::async_trait]
impl StorageInterface for RedisStorage {
    async fn init(&mut self) -> Result<()> {
        let mut conn = self.connection_pool.write().await;
        let _: () = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn commit_stats(&mut self, term: &str, tf: f32, df: u64) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let mut pipe = redis::pipe();
        pipe.cmd("HSET").arg(self.make_tf_key(term)).arg("default").arg(tf);
        pipe.cmd("SET").arg(self.make_df_key(term)).arg(df as usize);

        let _: () = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn commit_batch(&mut self, stats: &Bm25Stats) -> Result<()> {
        if stats.tf.is_empty() && stats.df.is_empty() {
            return Ok(());
        }

        let mut conn = self.get_connection().await?;
        let mut pipe = redis::pipe();

        for (term, tf) in &stats.tf {
            pipe.cmd("HSET")
                .arg(self.make_tf_key(term))
                .arg("default")
                .arg(*tf);
        }

        for (term, df) in &stats.df {
            pipe.cmd("SET")
                .arg(self.make_df_key(term))
                .arg(*df as usize);
        }

        let _: () = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let mut conn = self.get_connection().await?;

        let tf: Option<f32> = redis::cmd("HGET")
            .arg(self.make_tf_key(term))
            .arg("default")
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        let df: Option<u64> = redis::cmd("GET")
            .arg(self.make_df_key(term))
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        if tf.is_none() && df.is_none() {
            return Ok(None);
        }

        let mut tf_map = HashMap::new();
        if let Some(tf_val) = tf {
            tf_map.insert(term.to_string(), tf_val);
        }

        let mut df_map = HashMap::new();
        if let Some(df_val) = df {
            df_map.insert(term.to_string(), df_val);
        }

        Ok(Some(Bm25Stats {
            tf: tf_map,
            df: df_map,
            total_docs: 0,
            avg_doc_length: 0.0,
        }))
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let mut conn = self.get_connection().await?;

        let df: Option<u64> = redis::cmd("GET")
            .arg(self.make_df_key(term))
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        Ok(df)
    }

    async fn get_tf(&self, term: &str, _doc_id: &str) -> Result<Option<f32>> {
        let mut conn = self.get_connection().await?;

        let tf: Option<f32> = redis::cmd("HGET")
            .arg(self.make_tf_key(term))
            .arg("default")
            .query_async(&mut conn)
            .await
            .unwrap_or(None);

        Ok(tf)
    }

    async fn remove_term(&mut self, term: &str) -> Result<()> {
        let mut conn = self.get_connection().await?;

        let _: () = redis::cmd("DEL")
            .arg(&[self.make_tf_key(term), self.make_df_key(term)])
            .query_async(&mut conn)
            .await
            .map_err(|e| Bm25Error::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        if !keys.is_empty() {
            let mut conn = self.get_connection().await?;
            let _: () = redis::cmd("DEL")
                .arg(keys.as_slice())
                .query_async(&mut conn)
                .await
                .map_err(|e| Bm25Error::StorageError(e.to_string()))?;
        }

        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        let pattern = format!("{}:*", self.key_prefix);
        let keys = self.scan_keys(&pattern).await?;

        let tf_pattern = format!("{}:tf:*", self.key_prefix);
        let tf_keys = self.scan_keys(&tf_pattern).await?;

        let df_pattern = format!("{}:df:*", self.key_prefix);
        let df_keys = self.scan_keys(&df_pattern).await?;

        let mut total_size = 0u64;
        for key in &keys {
            let mut conn = self.get_connection().await?;
            let size: Option<usize> = redis::cmd("STRLEN")
                .arg(key)
                .query_async(&mut conn)
                .await
                .unwrap_or(Some(0));
            total_size += size.unwrap_or(0) as u64;
        }

        Ok(StorageInfo {
            name: "RedisStorage".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            size: total_size,
            document_count: 0,
            term_count: df_keys.len(),
            is_connected: true,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        match self.get_connection().await {
            Ok(mut conn) => {
                let result: String = redis::cmd("PING")
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| Bm25Error::StorageError(e.to_string()))?;
                Ok(result == "PONG")
            }
            Err(_) => Ok(false),
        }
    }
}
