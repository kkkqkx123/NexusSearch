# Inversearch 存储模块架构设计

## 1. 概述

本文档描述 Inversearch 存储模块的完整架构设计，参考 BM25 服务的成功实践，建立分层清晰、职责明确的存储架构。

### 1.1 设计目标

1. **分层清晰**：建立 存储层 → 管理层 → 业务层 的三层架构
2. **职责分离**：存储实现、管理逻辑、业务操作各司其职
3. **易于扩展**：支持多种存储后端（File/Redis/WAL/ColdWarmCache）
4. **统一接口**：通过 StorageInterface 和 StorageManager 提供一致 API
5. **异步支持**：全面支持异步操作，适应现代 Rust 异步生态

### 1.2 当前问题

根据分析，当前 Inversearch 存储模块存在以下问题：

- ❌ 缺少 StorageManager 作为中间管理层
- ❌ 存储与业务逻辑（Index 操作）完全脱节
- ❌ StorageInterface 设计不完整，缺少关键方法
- ❌ 存储实例在服务中仅作为"摆设"，未参与实际业务
- ❌ 缺少统一的初始化和生命周期管理

### 1.3 参考架构（BM25）

BM25 服务的存储架构：

```
┌─────────────────────────────────────────┐
│         API Layer (gRPC/Embedded)       │
└────────────────┬────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│      Business Logic Layer (Core)        │
│  ┌─────────────────────────────────┐    │
│  │ IndexManager / SearchManager    │    │
│  └─────────────┬───────────────────┘    │
└────────────────┼────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│       Storage Manager Layer             │
│  ┌─────────────────────────────────┐    │
│  │ StorageManager                  │    │
│  │ MutableStorageManager           │    │
│  └─────────────┬───────────────────┘    │
└────────────────┼────────────────────────┘
                 │
┌────────────────▼────────────────────────┐
│         Storage Interface               │
│  ┌──────────┐ ┌──────────┐ ┌────────┐  │
│  │Tantivy   │ │Redis     │ │Other   │  │
│  │Storage   │ │Storage   │ │Storage │  │
│  └──────────┘ └──────────┘ └────────┘  │
└─────────────────────────────────────────┘
```

## 2. 新架构设计

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────┐
│                  Application Layer                      │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  │
│  │ gRPC Server  │  │Embedded API  │  │HTTP Server   │  │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘  │
└─────────┼─────────────────┼─────────────────┼──────────┘
          │                 │                 │
┌─────────▼─────────────────▼─────────────────▼──────────┐
│                   Business Logic Layer                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Index Operations  │ Search Operations           │  │
│  │  Document Manager  │ Batch Operations            │  │
│  └────────────────────┬─────────────────────────────┘  │
└───────────────────────┼────────────────────────────────┘
                        │
┌───────────────────────▼────────────────────────────────┐
│                  Storage Manager Layer                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  StorageManager (Read Operations)                │  │
│  │  MutableStorageManager (Write Operations)        │  │
│  │  - Lifecycle Management                          │  │
│  │  - Connection Pooling                            │  │
│  │  - Stats Collection                              │  │
│  │  - Error Handling                                │  │
│  └────────────────────┬─────────────────────────────┘  │
└───────────────────────┼────────────────────────────────┘
                        │
┌───────────────────────▼────────────────────────────────┐
│                  Storage Interface Layer                │
│  ┌──────────────────────────────────────────────────┐  │
│  │  StorageInterface (Trait)                        │  │
│  │  - mount/unmount                                 │  │
│  │  - commit/revert                                 │  │
│  │  - get/enrich                                    │  │
│  │  - health_check                                  │  │
│  └────────────────────┬─────────────────────────────┘  │
└───────────────────────┼────────────────────────────────┘
                        │
┌───────────────────────▼────────────────────────────────┐
│                  Storage Implementations                │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │ColdWarmCache │ │File Storage  │ │Redis Storage │   │
│  └──────────────┘ └──────────────┘ └──────────────┘   │
│  ┌──────────────┐ ┌──────────────┐                     │
│  │WAL Storage   │ │Memory Storage│                     │
│  └──────────────┘ └──────────────┘                     │
└─────────────────────────────────────────────────────────┘
```

### 2.2 核心组件职责

#### 2.21 StorageInterface（存储接口层）

**职责**：定义存储后端的标准接口

```rust
#[async_trait::async_trait]
pub trait StorageInterface: Send + Sync {
    // 生命周期管理
    async fn init(&mut self) -> Result<()>;
    async fn shutdown(&mut self) -> Result<()>;
    
    // 索引操作
    async fn mount(&mut self, index: &Index) -> Result<()>;
    async fn unmount(&mut self) -> Result<()>;
    async fn commit(&mut self, index: &Index, mode: CommitMode) -> Result<()>;
    async fn revert(&mut self, index: &mut Index) -> Result<()>;
    
    // 数据访问
    async fn get(&self, key: &str, ctx: Option<&str>) -> Result<SearchResults>;
    async fn enrich(&self, ids: &[DocId]) -> Result<EnrichedSearchResults>;
    async fn has(&self, id: DocId) -> Result<bool>;
    
    // 文档管理
    async fn get_document(&self, id: DocId) -> Result<Option<String>>;
    async fn store_document(&mut self, id: DocId, content: String) -> Result<()>;
    async fn remove_document(&mut self, id: DocId) -> Result<()>;
    
    // 批量操作
    async fn batch_commit(&mut self, changes: &[IndexChange]) -> Result<()>;
    async fn batch_remove(&mut self, ids: &[DocId]) -> Result<()>;
    
    // 管理操作
    async fn clear(&mut self) -> Result<()>;
    async fn info(&self) -> Result<StorageInfo>;
    async fn health_check(&self) -> Result<bool>;
}
```

#### 2.22 StorageManager（存储管理层）

**职责**：统一管理存储实例，提供高级抽象

```rust
/// 只读存储管理器
#[derive(Clone)]
pub struct StorageManager {
    storage: Arc<dyn StorageInterface>,
    config: StorageConfig,
    metrics: Arc<StorageMetrics>,
}

impl StorageManager {
    // 索引管理
    pub async fn mount_index(&self, index: &Index) -> Result<()>;
    pub async fn unmount_index(&self) -> Result<()>;
    pub async fn sync_index(&self, index: &Index) -> Result<()>;
    
    // 数据查询
    pub async fn search(&self, term: &str, options: SearchOptions) -> Result<SearchResults>;
    pub async fn get_documents(&self, ids: &[DocId]) -> Result<Vec<Option<String>>>;
    pub async fn enrich_results(&self, results: &SearchResults) -> Result<EnrichedSearchResults>;
    
    // 统计信息
    pub async fn get_stats(&self) -> Result<StorageStats>;
    pub async fn get_index_info(&self) -> Result<StorageInfo>;
    
    // 健康检查
    pub async fn health_check(&self) -> Result<HealthStatus>;
}

/// 可变存储管理器（支持写操作）
pub struct MutableStorageManager {
    storage: Arc<RwLock<Box<dyn StorageInterface>>>,
    config: StorageConfig,
    metrics: Arc<StorageMetrics>,
}

impl MutableStorageManager {
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

#### 2.23 业务逻辑层集成

**Index 模块集成存储**：

```rust
pub struct Index {
    // ... 现有字段 ...
    
    // 可选的存储管理器（通过配置启用）
    #[cfg(feature = "storage")]
    storage: Option<Arc<MutableStorageManager>>,
}

impl Index {
    /// 创建带存储的索引
    #[cfg(feature = "storage")]
    pub fn with_storage(options: IndexOptions, storage: MutableStorageManager) -> Result<Self> {
        let mut index = Self::new(options)?;
        index.storage = Some(Arc::new(storage));
        Ok(index)
    }
    
    /// 添加文档（带存储同步）
    #[cfg(feature = "storage")]
    pub async fn add_async(&mut self, id: DocId, content: &str) -> Result<()> {
        // 1. 操作内存索引
        self.add(id, content, false)?;
        
        // 2. 同步到存储
        if let Some(ref storage) = self.storage {
            storage.add_document(id, content).await?;
        }
        
        Ok(())
    }
    
    /// 移除文档（带存储同步）
    #[cfg(feature = "storage")]
    pub async fn remove_async(&mut self, id: DocId) -> Result<()> {
        // 1. 操作内存索引
        self.remove(id, false)?;
        
        // 2. 从存储删除
        if let Some(ref storage) = self.storage {
            storage.remove_document(id).await?;
        }
        
        Ok(())
    }
    
    /// 搜索（支持从存储加载）
    pub async fn search_async(&self, query: &str, options: SearchOptions) -> Result<SearchResult> {
        // 1. 尝试从缓存/存储获取
        #[cfg(feature = "storage")]
        if let Some(ref storage) = self.storage {
            if options.use_storage {
                return storage.search(query, options).await;
            }
        }
        
        // 2.  fallback 到内存搜索
        Ok(self.search(query, options)?)
    }
}
```

### 2.3 数据流设计

#### 2.31 写入流程

```
用户请求
   │
   ▼
┌─────────────────┐
│  Index.add()    │ 同步写入内存
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ StorageManager  │ 异步写入存储
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│  WAL  │ │ Cache │ 先写 WAL，再异步刷盘
└───────┘ └───────┘
```

#### 2.32 读取流程

```
用户查询
   │
   ▼
┌─────────────────┐
│ Search Cache    │ 一级缓存（内存）
└────────┬────────┘
         │ Miss
         ▼
┌─────────────────┐
│ Hot Cache       │ 二级缓存（冷热分离）
└────────┬────────┘
         │ Miss
         ▼
┌─────────────────┐
│ StorageManager  │ 从存储加载
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
    ▼         ▼
┌───────┐ ┌───────┐
│ Redis │ │ File  │ 根据配置选择
└───────┘ └───────┘
```

### 2.4 错误处理策略

```rust
pub enum StorageError {
    /// 连接错误
    ConnectionError(String),
    /// 序列化错误
    SerializationError(String),
    /// 数据不一致
    DataInconsistencyError(String),
    /// 超时错误
    TimeoutError(Duration),
    /// 存储已满
    StorageFullError,
    /// 不支持的操作
    UnsupportedOperation(String),
}

// 错误处理原则
// 1. 存储错误不应影响内存索引的完整性
// 2. 写失败应记录 WAL 以便恢复
// 3. 读失败应 fallback 到其他存储或返回空
// 4. 所有错误应记录详细的上下文信息
```

### 2.5 性能优化设计

#### 2.51 批量提交

```rust
pub struct BatchCommitter {
    changes: Vec<IndexChange>,
    max_batch_size: usize,
    flush_interval: Duration,
}

impl BatchCommitter {
    /// 累积变更，达到阈值后批量提交
    pub async fn add_change(&mut self, change: IndexChange) {
        self.changes.push(change);
        
        if self.changes.len() >= self.max_batch_size {
            self.flush().await;
        }
    }
    
    /// 定时刷新
    pub async fn start_background_flush(&self) -> JoinHandle<()> {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(self.flush_interval).await;
                self.flush().await;
            }
        })
    }
}
```

#### 2.52 读写分离

```rust
pub struct ReadWriteStorage {
    read_storage: Arc<dyn StorageInterface>,  // Redis 等快速存储
    write_storage: Arc<dyn StorageInterface>, // File 等持久化存储
}

impl StorageInterface for ReadWriteStorage {
    async fn get(&self, key: &str) -> Result<SearchResults> {
        // 读操作走快速存储
        self.read_storage.get(key).await
    }
    
    async fn commit(&mut self, index: &Index) -> Result<()> {
        // 写操作走持久化存储
        self.write_storage.commit(index).await
    }
}
```

## 3. 特性标志设计

```toml
[features]
# 存储后端选择
default = ["storage-cold-warm-cache"]
storage-cold-warm-cache = []  # 冷热缓存（默认）
storage-file = []              # 文件存储
storage-redis = ["dep:redis"]  # Redis 存储
storage-wal = []               # WAL 预写日志

# 功能特性
storage = []                   # 启用存储支持
storage-async = ["tokio"]      # 异步存储操作
storage-metrics = ["metrics"]  # 存储指标收集
storage-compression = ["zstd"] # 压缩支持
```

## 4. 配置设计

```toml
[storage]
enabled = true
backend = "cold-warm-cache"  # cold-warm-cache | file | redis | wal

[storage.cold-warm-cache]
hot_cache_size = "1GB"
warm_cache_size = "4GB"
flush_interval_secs = 60
compression = true

[storage.file]
base_path = "./data/inversearch"
compression = true
flush_interval_secs = 300

[storage.redis]
url = "redis://localhost:6379"
pool_size = 10
key_prefix = "inversearch:"
ttl_secs = 3600

[storage.wal]
base_path = "./data/wal"
max_wal_size_mb = 512
checkpoint_interval_secs = 300
compression = true
```

## 5. 监控指标

```rust
pub struct StorageMetrics {
    // 操作计数
    pub reads: AtomicU64,
    pub writes: AtomicU64,
    pub deletes: AtomicU64,
    
    // 延迟统计
    pub read_latency_ns: Histogram,
    pub write_latency_ns: Histogram,
    
    // 缓存统计
    pub cache_hits: AtomicU64,
    pub cache_misses: AtomicU64,
    
    // 存储大小
    pub storage_size_bytes: AtomicU64,
    pub document_count: AtomicU64,
}

impl StorageMetrics {
    pub fn export_prometheus(&self) -> String {
        // 导出 Prometheus 格式指标
    }
}
```

## 6. 测试策略

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_storage_manager_commit() {
        let storage = MockStorage::new();
        let manager = StorageManager::new(storage);
        
        // 测试提交逻辑
    }
}
```

### 6.2 集成测试

```rust
#[cfg(test)]
mod integration_tests {
    #[tokio::test]
    async fn test_end_to_end_with_storage() {
        // 测试完整的写入→存储→读取流程
    }
}
```

### 6.3 性能测试

```rust
#[cfg(test)]
mod bench {
    #[bench]
    fn bench_storage_write(b: &mut Bencher) {
        // 测试存储写入性能
    }
}
```

## 7. 迁移路径

### Phase 1: 基础架构（1-2 周）
- [ ] 创建 StorageManager 和 MutableStorageManager
- [ ] 完善 StorageInterface trait
- [ ] 添加单元测试

### Phase 2: 模块集成（2-3 周）
- [ ] 集成到 Index 模块
- [ ] 集成到 Document 模块
- [ ] 集成到 Search 模块

### Phase 3: 服务集成（1-2 周）
- [ ] 集成到 gRPC 服务
- [ ] 集成到 Embedded API
- [ ] 添加配置支持

### Phase 4: 优化与完善（1-2 周）
- [ ] 性能优化（批量、缓存）
- [ ] 监控指标
- [ ] 文档完善

## 8. 风险与缓解

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| 性能下降 | 高 | 异步操作、批量提交、缓存优化 |
| 数据不一致 | 高 | WAL 保证、定期校验 |
| 复杂度增加 | 中 | 清晰分层、充分文档、单元测试 |
| 向后兼容 | 中 | 特性标志、渐进式迁移 |

## 9. 总结

新架构通过引入 StorageManager 层，解决了当前存储模块与业务逻辑脱节的问题，建立了清晰的分层架构。参考 BM25 的成功实践，确保了设计的可行性和完整性。
