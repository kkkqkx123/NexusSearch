# Inversearch 存储模块迁移方案

## 1. 迁移概述

本文档详细描述如何将 Inversearch 的存储模块从当前架构迁移到新的分层架构。迁移将分阶段进行，确保向后兼容性和系统稳定性。

### 1.1 当前状态

**现有架构：**
```
storage/
├── common/              # 公共组件（保留）
├── base.rs              # 存储基类（废弃）
├── utils.rs             # 工具函数（保留到 common）
├── file.rs              # 文件存储（重构）
├── redis.rs             # Redis 存储（重构）
├── wal.rs               # WAL 模块（重构）
├── wal_storage.rs       # WAL 存储（重构）
├── cold_warm_cache/     # 冷热缓存（保留并改进）
├── memory.rs            # 内存存储（重构）
└── factory.rs           # 存储工厂（重构）
```

**问题：**
- ❌ 缺少 StorageManager 层
- ❌ StorageInterface 不完整
- ❌ 存储与业务逻辑脱节
- ❌ 代码组织不够清晰

### 1.2 目标架构

**新架构：**
```
storage/
├── common/              # 公共组件（增强）
│   ├── trait.rs         # 完整的 StorageInterface
│   ├── types.rs         # 类型定义
│   ├── config.rs        # 配置（新增）
│   ├── error.rs         # 错误类型（新增）
│   └── metrics.rs       # 指标（新增）
├── manager/             # 存储管理（新增核心）
│   ├── storage_manager.rs
│   └── mutable_manager.rs
├── cold_warm_cache/     # 冷热缓存（改进）
├── file/                # 文件存储（重构）
├── redis/               # Redis 存储（重构）
├── wal/                 # WAL（重构）
├── memory/              # 内存存储（重构）
├── factory/             # 工厂（重构）
└── integration/         # 集成支持（新增）
```

## 2. 迁移原则

### 2.1 向后兼容

1. **保持现有 API**：现有导出路径继续可用
2. **渐进式迁移**：新旧代码可以共存
3. **特性标志**：通过 feature flags 控制新功能

### 2.2 分阶段进行

1. **Phase 1**：新增核心组件，不影响现有代码
2. **Phase 2**：重构现有模块，保持接口不变
3. **Phase 3**：集成到业务模块，启用新功能
4. **Phase 4**：清理废弃代码，完成迁移

### 2.3 测试保障

1. **单元测试**：每个新组件都有完整测试
2. **集成测试**：确保模块间协作正常
3. **回归测试**：保证现有功能不受影响

## 3. 详细迁移步骤

### Phase 1: 基础架构搭建（1-2 周）

#### Step 1.1: 创建新目录结构

```bash
# 创建新目录
mkdir -p src/storage/manager
mkdir -p src/storage/integration
mkdir -p src/storage/common/config
mkdir -p src/storage/common/error
mkdir -p src/storage/common/metrics

# 重构单文件模块为子模块
mkdir -p src/storage/file
mkdir -p src/storage/redis
mkdir -p src/storage/wal
mkdir -p src/storage/memory
mkdir -p src/storage/factory
```

#### Step 1.2: 完善 StorageInterface

**文件**: `src/storage/common/trait.rs`

**变更**:
```rust
// 当前（不完整）
pub trait StorageInterface {
    async fn mount(&mut self, index: &Index) -> Result<()>;
    async fn commit(&mut self, index: &Index, replace: bool, append: bool) -> Result<()>;
    async fn get(...) -> Result<SearchResults>;
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    async fn has(&self, id: DocId) -> Result<bool>;
    async fn remove(&mut self, ids: &[DocId]) -> Result<()>;
    async fn clear(&mut self) -> Result<()>;
    async fn info(&self) -> Result<StorageInfo>;
}

// 新（完整）
pub trait StorageInterface: Send + Sync {
    // 生命周期管理（新增）
    async fn init(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    
    // 索引操作（改进）
    async fn mount(&mut self, index: &Index) -> Result<()>;
    async fn unmount(&mut self) -> Result<()>;
    async fn commit(&mut self, index: &Index, mode: CommitMode) -> Result<()>;
    async fn revert(&mut self, index: &mut Index) -> Result<()>;
    
    // 数据查询（增强）
    async fn get(&self, key: &str, ctx: Option<&str>, options: SearchOptions) -> Result<SearchResults>;
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    async fn has(&self, id: DocId) -> Result<bool>;
    
    // 文档管理（新增）
    async fn get_document(&self, id: DocId) -> Result<Option<String>>;
    async fn store_document(&mut self, id: DocId, content: String) -> Result<()>;
    async fn remove_document(&mut self, id: DocId) -> Result<()>;
    
    // 批量操作（新增）
    async fn batch_commit(&mut self, changes: &[IndexChange]) -> Result<()>;
    async fn batch_remove(&mut self, ids: &[DocId]) -> Result<()>;
    
    // 管理操作（增强）
    async fn clear(&mut self) -> Result<()>;
    async fn info(&self) -> Result<StorageInfo>;
    async fn health_check(&self) -> Result<bool>;
    async fn backup(&self, path: &Path) -> Result<BackupInfo>;
    async fn restore(&self, backup: &BackupInfo) -> Result<()>;
}
```

**迁移策略**: 
- 新增方法提供默认实现（返回错误）
- 现有存储实现只需实现必需方法
- 渐进式实现所有方法

#### Step 1.3: 创建 StorageManager

**文件**: `src/storage/manager/storage_manager.rs`

```rust
/// 只读存储管理器
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<dyn StorageInterface>,
    config: StorageConfig,
    metrics: Arc<StorageMetrics>,
}

impl StorageManager {
    pub fn new(storage: Arc<dyn StorageInterface>, config: StorageConfig) -> Self;
    
    // 索引管理
    pub async fn mount_index(&self, index: &Index) -> Result<()>;
    pub async fn sync_index(&self, index: &Index) -> Result<()>;
    
    // 数据查询
    pub async fn search(&self, term: &str, options: SearchOptions) -> Result<SearchResults>;
    pub async fn get_document(&self, id: DocId) -> Result<Option<String>>;
    pub async fn enrich_results(&self, results: &SearchResults) -> Result<EnrichedSearchResults>;
    
    // 统计信息
    pub async fn get_stats(&self) -> Result<StorageStats>;
    pub async fn get_info(&self) -> Result<StorageInfo>;
    
    // 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus>;
}
```

**文件**: `src/storage/manager/mutable_manager.rs`

```rust
/// 可变存储管理器
pub struct MutableStorageManager {
    storage: Arc<RwLock<Box<dyn StorageInterface>>>,
    config: StorageConfig,
    metrics: Arc<StorageMetrics>,
}

impl MutableStorageManager {
    pub fn new(storage: Box<dyn StorageInterface>, config: StorageConfig) -> Self;
    
    // 文档写入
    pub async fn add_document(&self, id: DocId, content: &str) -> Result<()>;
    pub async fn remove_document(&self, id: DocId) -> Result<()>;
    pub async fn update_document(&self, id: DocId, content: &str) -> Result<()>;
    
    // 批量操作
    pub async fn batch_add(&self, documents: &[(DocId, String)]) -> Result<()>;
    pub async fn batch_remove(&self, ids: &[DocId]) -> Result<()>;
    pub async fn batch_update(&self, documents: &[(DocId, String)]) -> Result<()>;
    
    // 索引同步
    pub async fn commit_changes(&self, changes: &[IndexChange]) -> Result<()>;
    pub async fn sync_with_index(&self, index: &Index) -> Result<()>;
    
    // 管理操作
    pub async fn clear_all(&self) -> Result<()>;
    pub async fn backup(&self, path: &Path) -> Result<BackupInfo>;
    pub async fn restore(&self, backup: &BackupInfo) -> Result<()>;
}
```

#### Step 1.4: 创建集成模块

**文件**: `src/storage/integration/mod.rs`

```rust
//! 存储集成模块
//!
//! 提供存储模块与核心业务模块的集成支持

pub mod index;      // 与 Index 集成
pub mod document;   // 与 Document 集成
pub mod search;     // 与 Search 集成
```

**文件**: `src/storage/integration/index.rs`

```rust
//! Index 模块存储集成

use crate::storage::MutableStorageManager;
use crate::Index;

/// Index 存储集成扩展
pub trait IndexStorageExt {
    /// 创建带存储的索引
    async fn with_storage(
        options: IndexOptions,
        storage: MutableStorageManager
    ) -> Result<Self>;
    
    /// 添加文档（带存储同步）
    async fn add_async(&mut self, id: DocId, content: &str) -> Result<()>;
    
    /// 移除文档（带存储同步）
    async fn remove_async(&mut self, id: DocId) -> Result<()>;
    
    /// 搜索（支持从存储加载）
    async fn search_async(&self, query: &str, options: SearchOptions) -> Result<SearchResult>;
}

impl IndexStorageExt for Index {
    // 实现细节
}
```

### Phase 2: 重构现有模块（2-3 周）

#### Step 2.1: 重构 File 存储

**当前**: `src/storage/file.rs` (单文件)

**目标**:
```
src/storage/file/
├── mod.rs           # 模块入口，重新导出
├── storage.rs       # FileStorage 实现
├── config.rs        # FileStorageConfig
└── utils.rs         # 文件操作工具
```

**mod.rs**:
```rust
mod storage;
mod config;
mod utils;

pub use storage::FileStorage;
pub use config::FileStorageConfig;
```

**storage.rs** (实现新的 StorageInterface):
```rust
use crate::storage::common::r#trait::StorageInterface;

pub struct FileStorage {
    base_path: PathBuf,
    compression: bool,
    // ...
}

#[async_trait::async_trait]
impl StorageInterface for FileStorage {
    async fn init(&mut self) -> Result<()> {
        // 创建目录
        std::fs::create_dir_all(&self.base_path)?;
        Ok(())
    }
    
    async fn commit(&mut self, index: &Index, mode: CommitMode) -> Result<()> {
        // 实现提交逻辑
    }
    
    // ... 实现其他方法
}
```

**迁移步骤**:
1. 创建新目录结构
2. 将现有代码移动到 `storage.rs`
3. 实现新增的 StorageInterface 方法
4. 更新 `mod.rs` 导出
5. 更新 `storage/mod.rs` 中的引用
6. 运行测试确保功能正常

#### Step 2.2: 重构 Redis 存储

**当前**: `src/storage/redis.rs`

**目标**:
```
src/storage/redis/
├── mod.rs
├── storage.rs       # RedisStorage 实现
├── config.rs        # RedisStorageConfig
├── connection.rs    # 连接池管理
└── keys.rs          # Redis 键命名
```

**关键改进**:
```rust
// connection.rs
pub struct RedisConnectionPool {
    pool: redis::aio::ConnectionManager,
    config: RedisStorageConfig,
}

impl RedisConnectionPool {
    pub async fn new(config: RedisStorageConfig) -> Result<Self>;
    pub async fn get(&self) -> Result<Connection>;
}

// keys.rs - 统一的键命名规范
pub struct RedisKeyBuilder {
    prefix: String,
}

impl RedisKeyBuilder {
    pub fn index_key(&self, index_name: &str) -> String;
    pub fn document_key(&self, doc_id: DocId) -> String;
    pub fn term_key(&self, term: &str) -> String;
}
```

#### Step 2.3: 重构 WAL 模块

**当前**: 
- `src/storage/wal.rs` (WAL 管理)
- `src/storage/wal_storage.rs` (WAL 存储实现)

**目标**:
```
src/storage/wal/
├── mod.rs
├── wal_manager.rs   # WALManager (重命名)
├── config.rs        # WALConfig
├── entry.rs         # WAL 条目
├── writer.rs        # WAL 写入器
└── reader.rs        # WAL 读取器

src/storage/wal_storage/
├── mod.rs
├── storage.rs       # WALStorage (重命名)
└── checkpoint.rs    # 检查点机制
```

**关键改进**:
```rust
// entry.rs - 统一的 WAL 条目类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WALEntry {
    Add {
        doc_id: DocId,
        content: String,
        timestamp: u64,
    },
    Remove {
        doc_id: DocId,
        timestamp: u64,
    },
    Update {
        doc_id: DocId,
        content: String,
        timestamp: u64,
    },
    Checkpoint {
        snapshot_id: String,
        timestamp: u64,
    },
}

// writer.rs - 批量写入优化
pub struct WALWriter {
    buffer: Vec<WALEntry>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl WALWriter {
    pub async fn write(&mut self, entry: WALEntry) -> Result<()>;
    pub async fn flush(&mut self) -> Result<()>;
}
```

#### Step 2.4: 重构 ColdWarmCache

**当前**: `src/storage/cold_warm_cache/`

**改进**:
```
src/storage/cold_warm_cache/
├── mod.rs
├── manager.rs       # ColdWarmCacheManager
├── config.rs        # ColdWarmCacheConfig
├── cache/
│   ├── mod.rs
│   ├── hot_cache.rs     # 热缓存（LRU）
│   ├── warm_cache.rs    # 温缓存（LFU）
│   └── policy.rs        # 淘汰策略
├── background.rs    # 后台刷新
└── metrics.rs       # 缓存指标
```

**关键改进**:
```rust
// cache/hot_cache.rs
pub struct HotCache {
    cache: LruCache<String, IndexData>,
    max_size: usize,
    metrics: CacheMetrics,
}

impl HotCache {
    pub fn get(&mut self, key: &str) -> Option<&IndexData>;
    pub fn put(&mut self, key: String, data: IndexData);
    pub fn remove(&mut self, key: &str) -> Option<IndexData>;
}

// cache/policy.rs
pub enum EvictionPolicy {
    LRU,  // Least Recently Used
    LFU,  // Least Frequently Used
    TTL,  // Time To Live
}
```

#### Step 2.5: 重构 Factory

**当前**: `src/storage/factory.rs`

**目标**:
```
src/storage/factory/
├── mod.rs           # StorageFactory
└── builder.rs       # StorageFactoryBuilder
```

**关键改进**:
```rust
// builder.rs - 构建器模式
pub struct StorageFactoryBuilder {
    config: StorageConfig,
    custom_initializers: HashMap<String, Box<dyn StorageInitializer>>,
}

impl StorageFactoryBuilder {
    pub fn new() -> Self;
    
    pub fn with_config(mut self, config: StorageConfig) -> Self;
    
    pub fn with_custom_backend(
        mut self,
        name: &str,
        initializer: Box<dyn StorageInitializer>
    ) -> Self;
    
    pub async fn build(self) -> Result<Arc<dyn StorageInterface>>;
}

// 支持自定义存储后端
pub trait StorageInitializer: Send + Sync {
    async fn initialize(&self, config: &StorageConfig) -> Result<Box<dyn StorageInterface>>;
}
```

### Phase 3: 集成到业务模块（2-3 周）

#### Step 3.1: 集成到 Index 模块

**文件**: `src/index/mod.rs`

**变更**:
```rust
// 当前
pub struct Index {
    pub map: KeystoreMap<String, Vec<DocId>>,
    pub ctx: KeystoreMap<String, Vec<DocId>>,
    // ...
}

// 新增（通过特性标志）
#[cfg(feature = "storage")]
pub struct Index {
    // 现有字段...
    
    #[cfg(feature = "storage")]
    storage: Option<Arc<MutableStorageManager>>,
}

// 新增方法
#[cfg(feature = "storage")]
impl Index {
    pub fn with_storage(
        options: IndexOptions,
        storage: MutableStorageManager
    ) -> Result<Self> {
        let mut index = Self::new(options)?;
        index.storage = Some(Arc::new(storage));
        Ok(index)
    }
    
    pub async fn add_async(&mut self, id: DocId, content: &str) -> Result<()> {
        // 1. 操作内存索引
        self.add(id, content, false)?;
        
        // 2. 同步到存储
        #[cfg(feature = "storage")]
        if let Some(ref storage) = self.storage {
            storage.add_document(id, content).await?;
        }
        
        Ok(())
    }
    
    pub async fn remove_async(&mut self, id: DocId) -> Result<()> {
        self.remove(id, false)?;
        
        #[cfg(feature = "storage")]
        if let Some(ref storage) = self.storage {
            storage.remove_document(id).await?;
        }
        
        Ok(())
    }
}
```

#### Step 3.2: 集成到 Document 模块

**文件**: `src/document/mod.rs`

**变更**:
```rust
#[cfg(feature = "storage")]
pub struct Document {
    // 现有字段...
    storage: Option<Arc<MutableStorageManager>>,
}

#[cfg(feature = "storage")]
impl Document {
    pub async fn add_async(&mut self, id: DocId, fields: &Fields) -> Result<()> {
        // 1. 更新内存字段索引
        self.add(id, fields)?;
        
        // 2. 同步到存储
        if let Some(ref storage) = self.storage {
            // 序列化字段数据
            let serialized = self.serialize_fields(fields)?;
            storage.add_document(id, serialized).await?;
        }
        
        Ok(())
    }
}
```

#### Step 3.3: 集成到 Search 模块

**文件**: `src/search/mod.rs`

**变更**:
```rust
#[cfg(feature = "storage")]
pub async fn search_with_storage(
    index: &Index,
    query: &str,
    options: SearchOptions,
    storage: Option<&StorageManager>
) -> Result<SearchResult> {
    // 1. 尝试从存储缓存获取
    if let Some(storage) = storage {
        if options.use_cache {
            if let Some(cached) = storage.search(query, options.clone()).await? {
                return Ok(cached);
            }
        }
    }
    
    // 2. Fallback 到内存搜索
    let result = search(index, query, options)?;
    
    // 3. 异步缓存到存储（不阻塞）
    if let Some(storage) = storage {
        let result_clone = result.clone();
        tokio::spawn(async move {
            let _ = storage.cache_result(query, &result_clone).await;
        });
    }
    
    Ok(result)
}
```

#### Step 3.4: 集成到 API 层

**文件**: `src/api/server/grpc.rs`

**变更**:
```rust
// 当前
pub struct InversearchService {
    index: Arc<RwLock<Index>>,
    #[allow(dead_code)]  // ❌ 未使用
    storage: Arc<dyn StorageInterface + Send + Sync>,
    config: Config,
}

// 新
pub struct InversearchService {
    index: Arc<RwLock<Index>>,
    storage_manager: Arc<StorageManager>,
    mutable_storage: Arc<MutableStorageManager>,
    config: Config,
}

impl InversearchService {
    pub async fn with_config(config: Config) -> Self {
        // 1. 创建存储
        let storage = StorageFactory::from_config(&config).await?;
        
        // 2. 创建管理器
        let storage_manager = StorageManager::new(
            storage.clone(),
            config.storage.clone()
        );
        
        let mutable_storage = MutableStorageManager::new(
            Box::new(storage),
            config.storage.clone()
        );
        
        // 3. 创建带存储的索引
        let index = Index::with_storage(
            IndexOptions::default(),
            mutable_storage.clone()
        ).await?;
        
        Self {
            index: Arc::new(RwLock::new(index)),
            storage_manager: Arc::new(storage_manager),
            mutable_storage: Arc::new(mutable_storage),
            config,
        }
    }
    
    // API 方法使用存储管理器
    pub async fn add_document(&self, id: DocId, content: String) -> Result<()> {
        let mut index = self.index.write().await;
        
        // 使用带存储同步的方法
        index.add_async(id, &content).await?;
        
        Ok(())
    }
}
```

### Phase 4: 清理与优化（1-2 周）

#### Step 4.1: 移除废弃代码

**移除的文件**:
- `src/storage/base.rs` (功能已整合到 manager)
- `src/storage/utils.rs` (移动到 common/utils.rs)

**更新的导出**:
```rust
// src/storage/mod.rs
// 移除废弃的导出
// pub use base::StorageBase;  // ❌ 废弃

// 保留向后兼容的导出
#[deprecated(since = "0.8.3", note = "Use StorageManager instead")]
pub use base::StorageBase;
```

#### Step 4.2: 更新文档

更新所有相关文档：
- `README.md`
- `docs/` 下的文档
- 代码注释

#### Step 4.3: 性能优化

1. **批量提交优化**
```rust
pub struct BatchCommitter {
    changes: Vec<IndexChange>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl BatchCommitter {
    pub async fn add(&mut self, change: IndexChange) {
        self.changes.push(change);
        
        if self.changes.len() >= self.max_batch_size {
            self.flush().await;
        }
    }
}
```

2. **缓存优化**
```rust
// 实现二级缓存
pub struct TieredCache {
    l1: LruCache<String, IndexData>,      // 热缓存（内存）
    l2: Arc<dyn StorageInterface>,        // 温缓存（存储）
}
```

#### Step 4.4: 完善测试

**单元测试**:
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_storage_manager_commit() {
        let storage = MockStorage::new();
        let manager = StorageManager::new(storage, config);
        
        // 测试提交逻辑
        manager.sync_index(&index).await.unwrap();
        
        // 验证
        assert!(manager.health_check().await.unwrap().is_healthy);
    }
}
```

**集成测试**:
```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_end_to_end_with_storage() {
        // 1. 创建带存储的索引
        let mut index = create_index_with_storage().await;
        
        // 2. 添加文档
        index.add_async(1, "test content").await.unwrap();
        
        // 3. 重启服务（模拟）
        let index2 = load_index_from_storage().await;
        
        // 4. 验证数据完整性
        assert!(index2.contains(1));
    }
}
```

## 4. 迁移时间表

| Phase | 任务 | 预计时间 | 优先级 |
|-------|------|----------|--------|
| Phase 1 | 基础架构搭建 | 1-2 周 | 高 |
| Phase 2 | 重构现有模块 | 2-3 周 | 高 |
| Phase 3 | 集成到业务模块 | 2-3 周 | 高 |
| Phase 4 | 清理与优化 | 1-2 周 | 中 |

**总计**: 6-10 周

## 5. 风险评估与缓解

### 风险 1: 性能下降

**影响**: 存储同步可能降低写入性能

**缓解措施**:
- 异步写入，不阻塞主流程
- 批量提交，减少 IO 次数
- 缓存优化，减少存储访问

### 风险 2: 数据不一致

**影响**: 内存和存储数据不一致

**缓解措施**:
- WAL 保证操作原子性
- 定期校验数据一致性
- 提供数据恢复工具

### 风险 3: 向后兼容性

**影响**: 现有代码可能无法编译

**缓解措施**:
- 保留原有 API 导出路径
- 使用特性标志控制新功能
- 提供详细的迁移指南

### 风险 4: 复杂度增加

**影响**: 代码复杂度提高，维护困难

**缓解措施**:
- 清晰的分层架构
- 完善的文档和注释
- 充分的单元测试

## 6. 成功标准

1. ✅ 所有存储实现实现完整的 StorageInterface
2. ✅ StorageManager 和 MutableStorageManager 正常工作
3. ✅ 业务模块（Index/Document/Search）成功集成存储
4. ✅ 所有单元测试和集成测试通过
5. ✅ 性能指标达到预期（写入延迟 < 100ms）
6. ✅ 文档完整，迁移指南清晰

## 7. 后续改进

迁移完成后，可以考虑以下改进：

1. **分布式存储支持**: 支持多节点存储同步
2. **增量备份**: 实现增量备份和恢复
3. **存储加密**: 支持数据加密存储
4. **多版本控制**: 支持索引版本管理
5. **智能缓存**: 基于 ML 的缓存预测

## 8. 总结

本迁移方案通过分阶段、渐进式的方式，将 Inversearch 的存储模块重构为清晰的分层架构。迁移过程中保持向后兼容性，确保系统稳定性。参考 BM25 的成功实践，建立完整的存储管理体系。
