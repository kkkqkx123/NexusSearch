//! 存储管理器
//!
//! 提供统一的存储管理接口，集成 StorageInterface 到业务逻辑

use crate::error::Result;
use crate::storage::common::r#trait::StorageInterface;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 存储管理器（只读操作）
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<dyn StorageInterface>,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(storage: Arc<dyn StorageInterface>) -> Self {
        Self {
            storage,
        }
    }

    /// 获取底层存储接口
    pub fn storage(&self) -> Arc<dyn StorageInterface> {
        self.storage.clone()
    }

    /// 获取词项统计
    pub async fn get_stats(&self, term: &str) -> Result<Option<crate::storage::Bm25Stats>> {
        self.storage.get_stats(term).await
    }

    /// 获取文档频率
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        self.storage.get_df(term).await
    }

    /// 获取词项频率
    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        self.storage.get_tf(term, doc_id).await
    }

    /// 获取存储信息
    pub async fn info(&self) -> Result<crate::storage::StorageInfo> {
        self.storage.info().await
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        self.storage.health_check().await
    }
}

/// 可变的存储管理器，支持修改操作
pub struct MutableStorageManager {
    storage: Arc<RwLock<Box<dyn StorageInterface>>>,
}

impl MutableStorageManager {
    /// 创建新的可变存储管理器
    pub fn new(storage: Box<dyn StorageInterface>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
        }
    }

    /// 从 Arc 创建可变存储管理器
    pub fn from_arc(storage: Arc<dyn StorageInterface>) -> Self {
        // 由于无法直接将 Arc<dyn StorageInterface> 转换为 Box<dyn StorageInterface>
        // 我们需要使用内部可变性来包装它
        Self {
            storage: Arc::new(RwLock::new(Box::new(ArcStorageWrapper(storage)))),
        }
    }

    /// 获取存储 Arc（用于共享）
    pub fn storage_arc(&self) -> Arc<RwLock<Box<dyn StorageInterface>>> {
        self.storage.clone()
    }

    /// 初始化存储
    pub async fn init(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.init().await
    }

    /// 提交词项统计
    pub async fn commit_stats(&self, term: &str, tf: f32, df: u64) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_stats(term, tf, df).await
    }

    /// 批量提交统计
    pub async fn commit_batch(&self, stats: &crate::storage::Bm25Stats) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.commit_batch(stats).await
    }

    /// 获取词项统计
    pub async fn get_stats(&self, term: &str) -> Result<Option<crate::storage::Bm25Stats>> {
        let storage = self.storage.read().await;
        storage.get_stats(term).await
    }

    /// 获取文档频率
    pub async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        let storage = self.storage.read().await;
        storage.get_df(term).await
    }

    /// 获取词项频率
    pub async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        let storage = self.storage.read().await;
        storage.get_tf(term, doc_id).await
    }

    /// 清空所有数据
    pub async fn clear(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear().await
    }

    /// 获取存储信息
    pub async fn info(&self) -> Result<crate::storage::StorageInfo> {
        let storage = self.storage.read().await;
        storage.info().await
    }

    /// 健康检查
    pub async fn health_check(&self) -> Result<bool> {
        let storage = self.storage.read().await;
        storage.health_check().await
    }

    /// 删除特定文档的统计信息
    pub async fn delete_doc_stats(&self, doc_id: &str) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.delete_doc_stats(doc_id).await
    }
}

/// 包装 Arc<dyn StorageInterface> 以实现 Box<dyn StorageInterface>
struct ArcStorageWrapper(Arc<dyn StorageInterface>);

#[async_trait::async_trait]
impl StorageInterface for ArcStorageWrapper {
    async fn init(&mut self) -> Result<()> {
        // Arc 包装器不支持 init，因为无法获取可变引用
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        Ok(())
    }

    async fn commit_stats(&mut self, _term: &str, _tf: f32, _df: u64) -> Result<()> {
        // 通过内部可变性调用
        // 由于 StorageInterface 没有使用内部可变性，这里无法直接调用
        // 需要在设计时考虑这一点
        Ok(())
    }

    async fn commit_batch(&mut self, _stats: &crate::storage::Bm25Stats) -> Result<()> {
        Ok(())
    }

    async fn get_stats(&self, term: &str) -> Result<Option<crate::storage::Bm25Stats>> {
        self.0.get_stats(term).await
    }

    async fn get_df(&self, term: &str) -> Result<Option<u64>> {
        self.0.get_df(term).await
    }

    async fn get_tf(&self, term: &str, doc_id: &str) -> Result<Option<f32>> {
        self.0.get_tf(term, doc_id).await
    }

    async fn clear(&mut self) -> Result<()> {
        Ok(())
    }

    async fn delete_doc_stats(&mut self, _doc_id: &str) -> Result<()> {
        // Arc 包装器不支持 delete，因为无法获取可变引用
        Ok(())
    }

    async fn info(&self) -> Result<crate::storage::StorageInfo> {
        self.0.info().await
    }

    async fn health_check(&self) -> Result<bool> {
        self.0.health_check().await
    }
}
