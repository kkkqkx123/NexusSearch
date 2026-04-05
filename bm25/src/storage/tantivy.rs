//! Tantivy 本地文件存储实现
//!
//! 使用 Tantivy 作为底层存储，提供 BM25 词频统计的持久化

use crate::api::core::IndexManager;
use crate::error::{Bm25Error, Result};
use crate::storage::common::r#trait::{Bm25Stats, StorageInterface};
use crate::storage::common::types::StorageInfo;
use std::collections::HashMap;
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

    async fn get_stats(&self, term: &str) -> Result<Option<Bm25Stats>> {
        let manager = self.get_index_manager()?;
        let manager = manager.read().await;
        let reader = manager.reader()?;
        let searcher = reader.searcher();
        
        // Get term from the content field
        let field = manager.schema().get_field("content").unwrap();
        let term_obj = tantivy::Term::from_field_text(field, term);
        
        // Get document frequency
        let doc_freq = searcher.doc_freq(&term_obj)?;
        let total_docs = searcher.num_docs();
        
        // Calculate average document length
        let avg_doc_length = if total_docs > 0 {
            let total_terms = searcher.num_docs() * 100; // Approximation
            total_terms as f32 / total_docs as f32
        } else {
            0.0
        };
        
        Ok(Some(Bm25Stats {
            tf: HashMap::new(), // TF is calculated per document during search
            df: HashMap::from([(term.to_string(), doc_freq as u64)]),
            total_docs: total_docs as u64,
            avg_doc_length,
        }))
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let manager = self.get_index_manager()?;
        let manager = manager.read().await;
        let reader = manager.reader()?;
        let searcher = reader.searcher();
        
        let field = manager.schema().get_field("content").unwrap();
        let term_obj = tantivy::Term::from_field_text(field, term);
        
        let doc_freq = searcher.doc_freq(&term_obj)?;
        Ok(Some(doc_freq as u64))
    }

    async fn get_tf(&self, term: &str, _doc_id: &str) -> Result<Option<f32>> {
        // TF is calculated during search time in Tantivy
        // This is a simplified implementation
        let manager = self.get_index_manager()?;
        let manager = manager.read().await;
        let reader = manager.reader()?;
        let searcher = reader.searcher();
        
        let field = manager.schema().get_field("content").unwrap();
        let term_obj = tantivy::Term::from_field_text(field, term);
        
        let doc_freq = searcher.doc_freq(&term_obj)?;
        let total_docs = searcher.num_docs();
        
        // Simple TF calculation (in real BM25, this is more complex)
        if doc_freq > 0 && total_docs > 0 {
            let tf = (doc_freq as f32) / (total_docs as f32);
            Ok(Some(tf))
        } else {
            Ok(Some(0.0))
        }
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
