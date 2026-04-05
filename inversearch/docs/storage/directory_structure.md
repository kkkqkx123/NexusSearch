# Inversearch 存储模块目录结构设计

## 1. 目标目录结构

```
inversearch/src/storage/
├── mod.rs                    # 模块入口，重新导出公共 API
├── README.md                 # 模块使用说明
│
├── common/                   # 公共组件（所有存储实现共享）
│   ├── mod.rs
│   ├── trait.rs              # StorageInterface trait 定义
│   ├── types.rs              # 共享类型定义
│   ├── config.rs             # 存储配置类型
│   ├── error.rs              # 存储错误类型
│   ├── metrics.rs            # 性能指标收集
│   └── utils.rs              # 工具函数
│
├── manager/                  # 存储管理层（新增核心模块）
│   ├── mod.rs
│   ├── storage_manager.rs    # StorageManager（只读操作）
│   ├── mutable_manager.rs    # MutableStorageManager（写操作）
│   ├── builder.rs            # StorageManager 构建器
│   └── options.rs            # 管理器配置选项
│
├── cold_warm_cache/          # 冷热缓存存储实现（默认）
│   ├── mod.rs
│   ├── manager.rs            # ColdWarmCacheManager
│   ├── config.rs             # 冷热缓存配置
│   ├── cache/
│   │   ├── hot_cache.rs      # 热缓存实现
│   │   ├── warm_cache.rs     # 温缓存实现
│   │   └── policy.rs         # 缓存淘汰策略
│   ├── background.rs         # 后台刷新任务
│   └── metrics.rs            # 缓存指标
│
├── file/                     # 文件存储实现（重构）
│   ├── mod.rs
│   ├── storage.rs            # FileStorage 实现
│   ├── config.rs             # 文件存储配置
│   └── utils.rs              # 文件操作工具
│
├── redis/                    # Redis 存储实现（重构）
│   ├── mod.rs
│   ├── storage.rs            # RedisStorage 实现
│   ├── config.rs             # Redis 存储配置
│   ├── connection.rs         # 连接池管理
│   └── keys.rs               # Redis 键命名规范
│
├── wal/                      # WAL 预写日志实现（重构）
│   ├── mod.rs
│   ├── wal_manager.rs        # WALManager（重命名）
│   ├── config.rs             # WAL 配置
│   ├── entry.rs              # WAL 条目类型
│   ├── writer.rs             # WAL 写入器
│   └── reader.rs             # WAL 读取器
│
├── wal_storage/              # WAL 存储实现（重构）
│   ├── mod.rs
│   ├── storage.rs            # WALStorage 实现（重命名）
│   └── checkpoint.rs         # 检查点机制
│
├── memory/                   # 内存存储实现（测试用）
│   ├── mod.rs
│   └── storage.rs            # MemoryStorage 实现
│
├── factory/                  # 存储工厂（重构）
│   ├── mod.rs
│   └── builder.rs            # 工厂构建器
│
└── integration/              # 集成支持（新增）
    ├── mod.rs
    ├── index.rs              # 与 Index 模块集成
    ├── document.rs           # 与 Document 模块集成
    └── search.rs             # 与 Search 模块集成
```

## 2. 与 BM25 存储目录对比

```
bm25/src/storage/
├── mod.rs
├── common/
│   ├── mod.rs
│   ├── trait.rs              # ✓ 参考
│   └── types.rs              # ✓ 参考
├── tantivy.rs                # Tantivy 存储实现
├── redis.rs                  # Redis 存储实现
├── factory.rs                # ✓ 参考
└── manager.rs                # ✓ 重点参考
```

## 3. 核心文件详细说明

### 3.1 `mod.rs` - 模块入口

```rust
//! 存储模块
//!
//! 提供 Inversearch 的持久化存储支持，包括：
//! - 存储接口抽象（StorageInterface）
//! - 存储管理器（StorageManager, MutableStorageManager）
//! - 多种存储实现（File/Redis/WAL/ColdWarmCache）
//!
//! ## 架构
//!
//! ```text
//! Business Logic
//!       ↓
//! StorageManager
//!       ↓
//! StorageInterface
//!       ↓
//! Storage Implementations
//! ```
//!
//! ## 使用示例
//!
//! ```rust
//! use inversearch::storage::{StorageManager, MutableStorageManager};
//! use inversearch::storage::factory::StorageFactory;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // 创建存储
//! let config = StorageConfig::default();
//! let storage = StorageFactory::create(config).await?;
//!
//! // 创建管理器
//! let manager = StorageManager::new(storage);
//!
//! // 使用管理器
//! let info = manager.info().await?;
//! # Ok(())
//! # }
//! ```

// 公共组件
pub mod common;

// 存储管理层（新增）
pub mod manager;

// 存储实现
#[cfg(feature = "store-cold-warm-cache")]
pub mod cold_warm_cache;

#[cfg(feature = "store-file")]
pub mod file;

#[cfg(feature = "store-redis")]
pub mod redis;

#[cfg(feature = "store-wal")]
pub mod wal;

#[cfg(feature = "store-wal")]
pub mod wal_storage;

// 测试用内存存储
pub mod memory;

// 存储工厂
pub mod factory;

// 集成支持（新增）
#[cfg(feature = "storage")]
pub mod integration;

// 重新导出常用类型
pub use common::{
    config::StorageConfig,
    error::StorageError,
    metrics::StorageMetrics,
    r#trait::{StorageInterface, CommitMode},
    types::{StorageInfo, StorageStats, IndexData},
};

pub use manager::{StorageManager, MutableStorageManager};

#[cfg(feature = "store-cold-warm-cache")]
pub use cold_warm_cache::ColdWarmCacheManager;

#[cfg(feature = "store-file")]
pub use file::FileStorage;

#[cfg(feature = "store-redis")]
pub use redis::RedisStorage;

#[cfg(feature = "store-wal")]
pub use wal::WALManager;

#[cfg(feature = "store-wal")]
pub use wal_storage::WALStorage;

pub use factory::StorageFactory;
```

### 3.2 `common/trait.rs` - 存储接口定义

```rust
//! 存储接口定义

use crate::error::Result;
use crate::r#type::{DocId, EnrichedSearchResults, SearchResults};
use crate::storage::common::types::{StorageInfo, IndexData};
use crate::Index;

/// 提交模式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CommitMode {
    /// 完全替换
    Replace,
    /// 追加模式
    Append,
    /// 增量更新
    Incremental,
}

/// 索引变更类型
#[derive(Debug, Clone)]
pub enum IndexChange {
    Add {
        doc_id: DocId,
        content: String,
    },
    Remove {
        doc_id: DocId,
    },
    Update {
        doc_id: DocId,
        content: String,
    },
}

/// 存储接口 - 核心 trait
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    // ========== 生命周期管理 ==========
    
    /// 初始化存储
    async fn init(&mut self) -> Result<()>;
    
    /// 关闭存储
    async fn shutdown(&mut self) -> Result<()>;
    
    // ========== 索引操作 ==========
    
    /// 挂载索引
    async fn mount(&mut self, index: &Index) -> Result<()>;
    
    /// 卸载索引
    async fn unmount(&mut self) -> Result<()>;
    
    /// 提交索引变更
    async fn commit(&mut self, index: &Index, mode: CommitMode) -> Result<()>;
    
    /// 回滚索引
    async fn revert(&mut self, index: &mut Index) -> Result<()>;
    
    // ========== 数据查询 ==========
    
    /// 获取术语结果
    async fn get(
        &self,
        key: &str,
        ctx: Option<&str>,
        options: SearchOptions,
    ) -> Result<SearchResults>;
    
    /// 富化搜索结果
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    
    /// 检查 ID 是否存在
    async fn has(&self, id: DocId) -> Result<bool>;
    
    // ========== 文档管理 ==========
    
    /// 获取文档内容
    async fn get_document(&self, id: DocId) -> Result<Option<String>>;
    
    /// 存储文档
    async fn store_document(&mut self, id: DocId, content: String) -> Result<()>;
    
    /// 删除文档
    async fn remove_document(&mut self, id: DocId) -> Result<()>;
    
    // ========== 批量操作 ==========
    
    /// 批量提交变更
    async fn batch_commit(&mut self, changes: &[IndexChange]) -> Result<()>;
    
    /// 批量删除
    async fn batch_remove(&mut self, ids: &[DocId]) -> Result<()>;
    
    // ========== 管理操作 ==========
    
    /// 清空所有数据
    async fn clear(&mut self) -> Result<()>;
    
    /// 获取存储信息
    async fn info(&self) -> Result<StorageInfo>;
    
    /// 健康检查
    async fn health_check(&self) -> Result<bool>;
    
    /// 备份数据
    async fn backup(&self, path: &Path) -> Result<BackupInfo> {
        // 默认实现：返回不支持
        Err(StorageError::UnsupportedOperation("backup".to_string()).into())
    }
    
    /// 恢复数据
    async fn restore(&self, backup: &BackupInfo) -> Result<()> {
        // 默认实现：返回不支持
        Err(StorageError::UnsupportedOperation("restore".to_string()).into())
    }
}
```

### 3.3 `manager/storage_manager.rs` - 存储管理器

```rust
//! 存储管理器（只读操作）

use crate::error::Result;
use crate::storage::{StorageInterface, StorageConfig, StorageMetrics};
use crate::r#type::{DocId, SearchOptions, SearchResults, EnrichedSearchResults};
use crate::storage::common::types::{StorageInfo, StorageStats};
use std::sync::Arc;

/// 只读存储管理器
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<dyn StorageInterface>,
    config: StorageConfig,
    metrics: Arc<StorageMetrics>,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(storage: Arc<dyn StorageInterface>, config: StorageConfig) -> Self {
        Self {
            storage,
            config,
            metrics: Arc::new(StorageMetrics::default()),
        }
    }
    
    /// 获取底层存储接口
    pub fn storage(&self) -> Arc<dyn StorageInterface> {
        self.storage.clone()
    }
    
    /// 获取配置
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
    
    /// 获取指标
    pub fn metrics(&self) -> Arc<StorageMetrics> {
        self.metrics.clone()
    }
    
    // ========== 索引管理 ==========
    
    /// 挂载索引
    pub async fn mount_index(&self, index: &crate::Index) -> Result<()> {
        let _timer = self.metrics.operation_timer("mount_index");
        
        // TODO: 实现索引挂载逻辑
        // 1. 序列化索引
        // 2. 存储到后端
        // 3. 记录元数据
        
        Ok(())
    }
    
    /// 卸载索引
    pub async fn unmount_index(&self) -> Result<()> {
        let _timer = self.metrics.operation_timer("unmount_index");
        
        // TODO: 实现索引卸载逻辑
        Ok(())
    }
    
    /// 同步索引
    pub async fn sync_index(&self, index: &crate::Index) -> Result<()> {
        let _timer = self.metrics.operation_timer("sync_index");
        
        // TODO: 实现索引同步逻辑
        Ok(())
    }
    
    // ========== 数据查询 ==========
    
    /// 搜索
    pub async fn search(&self, term: &str, options: SearchOptions) -> Result<SearchResults> {
        let _timer = self.metrics.operation_timer("search");
        self.metrics.record_read();
        
        self.storage.get(term, None, options).await
    }
    
    /// 获取文档
    pub async fn get_document(&self, id: DocId) -> Result<Option<String>> {
        let _timer = self.metrics.operation_timer("get_document");
        self.metrics.record_read();
        
        self.storage.get_document(id).await
    }
    
    /// 批量获取文档
    pub async fn get_documents(&self, ids: &[DocId]) -> Result<Vec<Option<String>>> {
        let _timer = self.metrics.operation_timer("get_documents");
        
        let mut results = Vec::with_capacity(ids.len());
        for &id in ids {
            results.push(self.storage.get_document(id).await?);
        }
        
        Ok(results)
    }
    
    /// 富化搜索结果
    pub async fn enrich_results(&self, results: &SearchResults) -> Result<EnrichedSearchResults> {
        let _timer = self.metrics.operation_timer("enrich_results");
        
        self.storage.enrich(results).await
    }
    
    // ========== 统计信息 ==========
    
    /// 获取存储统计
    pub async fn get_stats(&self) -> Result<StorageStats> {
        let info = self.storage.info().await?;
        
        Ok(StorageStats {
            document_count: info.document_count,
            storage_size: info.size,
            cache_hits: self.metrics.cache_hits(),
            cache_misses: self.metrics.cache_misses(),
            hit_rate: self.metrics.cache_hit_rate(),
        })
    }
    
    /// 获取存储信息
    pub async fn get_info(&self) -> Result<StorageInfo> {
        self.storage.info().await
    }
    
    // ========== 健康检查 ==========
    
    /// 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let is_healthy = self.storage.health_check().await?;
        
        Ok(HealthStatus {
            is_healthy,
            storage_type: self.config.backend.to_string(),
            last_check: chrono::Utc::now(),
        })
    }
}

/// 健康状态
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub storage_type: String,
    pub last_check: chrono::DateTime<chrono::Utc>,
}
```

### 3.4 `manager/mutable_manager.rs` - 可变存储管理器

```rust
//! 可变存储管理器（支持写操作）

use crate::error::Result;
use crate::storage::{StorageInterface, StorageConfig, StorageMetrics, IndexChange};
use crate::r#type::DocId;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 可变存储管理器
pub struct MutableStorageManager {
    storage: Arc<RwLock<Box<dyn StorageInterface>>>,
    config: StorageConfig,
    metrics: Arc<StorageMetrics>,
}

impl MutableStorageManager {
    /// 创建新的可变存储管理器
    pub fn new(storage: Box<dyn StorageInterface>, config: StorageConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            config,
            metrics: Arc::new(StorageMetrics::default()),
        }
    }
    
    /// 从 Arc 创建（用于兼容现有代码）
    pub fn from_arc(storage: Arc<dyn StorageInterface>, config: StorageConfig) -> Self {
        Self {
            storage: Arc::new(RwLock::new(Box::new(ArcStorageWrapper(storage)))),
            config,
            metrics: Arc::new(StorageMetrics::default()),
        }
    }
    
    // ========== 文档写入 ==========
    
    /// 添加文档
    pub async fn add_document(&self, id: DocId, content: &str) -> Result<()> {
        let _timer = self.metrics.operation_timer("add_document");
        self.metrics.record_write();
        
        let mut storage = self.storage.write().await;
        storage.store_document(id, content.to_string()).await?;
        
        Ok(())
    }
    
    /// 删除文档
    pub async fn remove_document(&self, id: DocId) -> Result<()> {
        let _timer = self.metrics.operation_timer("remove_document");
        self.metrics.record_delete();
        
        let mut storage = self.storage.write().await;
        storage.remove_document(id).await?;
        
        Ok(())
    }
    
    /// 更新文档
    pub async fn update_document(&self, id: DocId, content: &str) -> Result<()> {
        let _timer = self.metrics.operation_timer("update_document");
        self.metrics.record_write();
        
        let mut storage = self.storage.write().await;
        storage.remove_document(id).await?;
        storage.store_document(id, content.to_string()).await?;
        
        Ok(())
    }
    
    // ========== 批量操作 ==========
    
    /// 批量添加文档
    pub async fn batch_add(&self, documents: &[(DocId, String)]) -> Result<()> {
        let _timer = self.metrics.operation_timer("batch_add");
        
        let changes: Vec<IndexChange> = documents
            .iter()
            .map(|(id, content)| IndexChange::Add {
                doc_id: *id,
                content: content.clone(),
            })
            .collect();
        
        let mut storage = self.storage.write().await;
        storage.batch_commit(&changes).await?;
        
        Ok(())
    }
    
    /// 批量删除文档
    pub async fn batch_remove(&self, ids: &[DocId]) -> Result<()> {
        let _timer = self.metrics.operation_timer("batch_remove");
        
        let mut storage = self.storage.write().await;
        storage.batch_remove(ids).await?;
        
        Ok(())
    }
    
    /// 批量更新文档
    pub async fn batch_update(&self, documents: &[(DocId, String)]) -> Result<()> {
        let _timer = self.metrics.operation_timer("batch_update");
        
        let changes: Vec<IndexChange> = documents
            .iter()
            .map(|(id, content)| IndexChange::Update {
                doc_id: *id,
                content: content.clone(),
            })
            .collect();
        
        let mut storage = self.storage.write().await;
        storage.batch_commit(&changes).await?;
        
        Ok(())
    }
    
    // ========== 索引同步 ==========
    
    /// 提交索引变更
    pub async fn commit_changes(&self, changes: &[IndexChange]) -> Result<()> {
        let _timer = self.metrics.operation_timer("commit_changes");
        
        let mut storage = self.storage.write().await;
        storage.batch_commit(changes).await?;
        
        Ok(())
    }
    
    /// 与索引同步
    pub async fn sync_with_index(&self, index: &crate::Index) -> Result<()> {
        let _timer = self.metrics.operation_timer("sync_with_index");
        
        let mut storage = self.storage.write().await;
        storage.commit(index, CommitMode::Incremental).await?;
        
        Ok(())
    }
    
    // ========== 管理操作 ==========
    
    /// 清空所有数据
    pub async fn clear_all(&self) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.clear().await
    }
    
    /// 备份数据
    pub async fn backup(&self, path: &Path) -> Result<BackupInfo> {
        let storage = self.storage.read().await;
        storage.backup(path).await
    }
    
    /// 恢复数据
    pub async fn restore(&self, backup: &BackupInfo) -> Result<()> {
        let mut storage = self.storage.write().await;
        storage.restore(backup).await
    }
}

/// 包装 Arc<dyn StorageInterface> 以实现 Box<dyn StorageInterface>
struct ArcStorageWrapper(Arc<dyn StorageInterface>);

#[async_trait::async_trait]
impl StorageInterface for ArcStorageWrapper {
    async fn init(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
    
    async fn mount(&mut self, index: &Index) -> Result<()> {
        self.0.mount(index).await
    }
    
    // ... 其他方法类似委托
}
```

### 3.5 `factory/mod.rs` - 存储工厂

```rust
//! 存储工厂模块
//!
//! 提供工厂方法用于创建存储实例

use crate::config::{Config, StorageBackend};
use crate::error::Result;
use crate::storage::{StorageInterface, StorageConfig};
use std::sync::Arc;

pub mod builder;

/// 存储工厂
pub struct StorageFactory;

impl StorageFactory {
    /// 从配置创建存储
    pub async fn from_config(config: &Config) -> Result<Arc<dyn StorageInterface>> {
        Self::create(&config.storage).await
    }
    
    /// 创建存储实例
    pub async fn create(config: &StorageConfig) -> Result<Arc<dyn StorageInterface>> {
        if !config.enabled {
            return Self::create_cold_warm_cache().await;
        }
        
        match &config.backend {
            #[cfg(feature = "store-file")]
            StorageBackend::File => Self::create_file(config),
            
            #[cfg(feature = "store-redis")]
            StorageBackend::Redis => Self::create_redis(config).await,
            
            #[cfg(feature = "store-wal")]
            StorageBackend::Wal => Self::create_wal(config).await,
            
            StorageBackend::ColdWarmCache => Self::create_cold_warm_cache().await,
            
            #[allow(unreachable_patterns)]
            _ => Self::create_cold_warm_cache().await,
        }
    }
    
    /// 创建文件存储
    #[cfg(feature = "store-file")]
    fn create_file(config: &StorageConfig) -> Result<Arc<dyn StorageInterface>> {
        use crate::storage::file::FileStorage;
        
        let path = config
            .file
            .as_ref()
            .map(|c| c.base_path.clone())
            .unwrap_or_else(|| "./data".to_string());
        
        let storage = FileStorage::new(path);
        Ok(Arc::new(storage))
    }
    
    /// 创建 Redis 存储
    #[cfg(feature = "store-redis")]
    async fn create_redis(config: &StorageConfig) -> Result<Arc<dyn StorageInterface>> {
        use crate::storage::redis::{RedisStorage, RedisStorageConfig};
        
        let redis_config = config
            .redis
            .as_ref()
            .map(|c| RedisStorageConfig {
                url: c.url.clone(),
                pool_size: c.pool_size,
                ..Default::default()
            })
            .unwrap_or_default();
        
        let storage = RedisStorage::new(redis_config).await?;
        Ok(Arc::new(storage))
    }
    
    /// 创建 WAL 存储
    #[cfg(feature = "store-wal")]
    async fn create_wal(config: &StorageConfig) -> Result<Arc<dyn StorageInterface>> {
        use crate::storage::wal_storage::WALStorage;
        use crate::storage::wal::WALConfig;
        
        let wal_config = config
            .wal
            .as_ref()
            .map(|c| WALConfig {
                base_path: std::path::PathBuf::from(&c.base_path),
                max_wal_size: c.max_wal_size,
                compression: c.compression,
                snapshot_interval: c.snapshot_interval,
                ..Default::default()
            })
            .unwrap_or_default();
        
        let storage = WALStorage::new(wal_config).await?;
        Ok(Arc::new(storage))
    }
    
    /// 创建冷热缓存存储
    async fn create_cold_warm_cache() -> Result<Arc<dyn StorageInterface>> {
        #[cfg(feature = "store-cold-warm-cache")]
        {
            use crate::storage::cold_warm_cache::ColdWarmCacheManager;
            let manager = ColdWarmCacheManager::new().await?;
            Ok(manager as Arc<dyn StorageInterface>)
        }
        
        #[cfg(not(feature = "store-cold-warm-cache"))]
        {
            Err(StorageError::ConfigError(
                "No storage backend available".to_string()
            ).into())
        }
    }
}
```

## 4. 迁移步骤

### Step 1: 保留现有结构（向后兼容）

保持现有文件位置不变，确保现有代码继续工作：
- `storage/common/` - 保留
- `storage/cold_warm_cache/` - 保留
- `storage/file.rs` → `storage/file/`
- `storage/redis.rs` → `storage/redis/`
- `storage/wal.rs` → `storage/wal/`
- `storage/wal_storage.rs` → `storage/wal_storage/`
- `storage/memory.rs` → `storage/memory/`
- `storage/factory.rs` → `storage/factory/`

### Step 2: 新增核心模块

创建新目录和文件：
- `storage/manager/` - 新增 StorageManager
- `storage/integration/` - 新增集成支持

### Step 3: 重构现有模块

将单文件模块重构为子模块：
- `file.rs` → `file/mod.rs` + `file/storage.rs`
- `redis.rs` → `redis/mod.rs` + `redis/storage.rs`
- 其他类似

### Step 4: 更新导出

在 `storage/mod.rs` 中重新导出所有公共 API，保持向后兼容。

## 5. 文件组织原则

1. **单一职责**：每个文件只负责一个功能点
2. **清晰分层**：common → manager → implementations
3. **易于导航**：通过 mod.rs 统一导出
4. **向后兼容**：保留原有导出路径
5. **渐进迁移**：支持新旧代码并存

## 6. 注意事项

1. 所有存储实现必须实现 `StorageInterface` trait
2. 对外暴露的 API 通过 `StorageManager` 和 `MutableStorageManager`
3. 使用特性标志控制编译哪些存储后端
4. 所有异步操作必须使用 `async/await`
5. 错误处理统一使用 `Result<T, StorageError>`
