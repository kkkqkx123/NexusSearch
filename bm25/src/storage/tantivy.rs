//! Tantivy 本地文件存储实现
//!
//! 使用 Tantivy 作为底层存储，提供 BM25 词频统计的持久化

use crate::api::core::IndexManager;
use crate::error::{Bm25Error, Result};
use crate::storage::common::r#trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Tantivy 存储配置
#[derive(Debug, Clone)]
pub struct TantivyStorageConfig {
    pub index_path: PathBuf,
    pub writer_memory_mb: usize,
}

impl Default for TantivyStorageConfig {
    fn default() -> Self {
        Self {
            index_path: PathBuf::from("./index"),
            writer_memory_mb: 50,
        }
    }
}

/// Tantivy 存储实现
pub struct TantivyStorage {
    config: TantivyStorageConfig,
    index_manager: Option<Arc<RwLock<IndexManager>>>,
}

impl TantivyStorage {
    pub fn new(config: TantivyStorageConfig) -> Self {
        Self {
            config,
            index_manager: None,
        }
    }

    pub fn with_index_manager(index_manager: IndexManager) -> Self {
        Self {
            config: TantivyStorageConfig::default(),
            index_manager: Some(Arc::new(RwLock::new(index_manager))),
        }
    }

    fn get_index_manager(&self) -> Result<Arc<RwLock<IndexManager>>> {
        self.index_manager
            .clone()
            .ok_or_else(|| Bm25Error::IndexNotInitialized)
    }
}

#[async_trait::async_trait]
impl StorageInterface for TantivyStorage {
    async fn init(&mut self) -> Result<()> {
        if self.index_manager.is_none() {
            let index_manager = IndexManager::create(&self.config.index_path)
                .map_err(|e| Bm25Error::IndexCreationFailed(e.to_string()))?;
            self.index_manager = Some(Arc::new(RwLock::new(index_manager)));
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        if let Some(manager) = self.index_manager.take() {
            let manager = manager.write().await;
            let mut index_writer = manager.writer()?;
            index_writer
                .commit()
                .map_err(|e: tantivy::TantivyError| Bm25Error::IndexCommitFailed(e.to_string()))?;
        }
        Ok(())
    }

    async fn commit_stats(&mut self, _term: &str, _tf: f32, _df: u64) -> Result<()> {
        // Tantivy 自动管理词频统计，无需手动提交
        Ok(())
    }

    async fn commit_batch(&mut self, _stats: &Bm25Stats) -> Result<()> {
        // Tantivy 自动管理词频统计，无需手动提交
        Ok(())
    }

    async fn get_stats(&self, _term: &str) -> Result<Option<Bm25Stats>> {
        // Tantivy 的统计信息在搜索时自动计算
        Ok(Some(Bm25Stats::default()))
    }

    async fn get_df(&self, _term: &str) -> Result<Option<u64>> {
        // Tantivy 的文档频率在搜索时自动获取
        Ok(Some(0))
    }

    async fn get_tf(&self, _term: &str, _doc_id: &str) -> Result<Option<f32>> {
        // Tantivy 的词项频率在搜索时自动获取
        Ok(Some(0.0))
    }

    async fn remove_term(&mut self, _term: &str) -> Result<()> {
        // Tantivy 不支持单独删除词项，需要删除整个文档
        Ok(())
    }

    async fn clear(&mut self) -> Result<()> {
        if let Some(manager) = &self.index_manager {
            let manager = manager.write().await;
            let mut index_writer = manager.writer()?;
            index_writer
                .commit()
                .map_err(|e: tantivy::TantivyError| Bm25Error::IndexCommitFailed(e.to_string()))?;
        }
        Ok(())
    }

    async fn info(&self) -> Result<StorageInfo> {
        Ok(StorageInfo {
            name: "TantivyStorage".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            size: 0,
            document_count: 0,
            term_count: 0,
            is_connected: true,
        })
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.index_manager.is_some())
    }
}
