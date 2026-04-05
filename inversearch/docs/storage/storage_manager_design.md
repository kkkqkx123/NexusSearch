# StorageManager 设计方案

## 1. 概述

StorageManager 是 Inversearch 存储架构的核心管理层，负责统一管理存储实例，提供高级抽象，隔离业务逻辑与底层存储实现。

### 1.1 设计动机

**当前问题**:
- ❌ 缺少存储管理层，业务代码直接操作存储
- ❌ 存储实例生命周期管理混乱
- ❌ 缺少统一的错误处理和指标收集
- ❌ 存储切换困难，耦合严重

**解决方案**:
- ✅ 引入 StorageManager 作为中间层
- ✅ 统一管理存储生命周期
- ✅ 提供一致的错误处理和指标
- ✅ 通过接口抽象实现存储可插拔

### 1.2 核心职责

```
┌─────────────────────────────────────┐
│      Business Logic (Index)         │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│       StorageManager                │
│  - Lifecycle Management             │
│  - Connection Pooling               │
│  - Error Handling                   │
│  - Metrics Collection               │
│  - Caching                          │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│    StorageInterface (Trait)         │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│  Storage Implementations            │
│  - ColdWarmCache                    │
│  - File Storage                     │
│  - Redis Storage                    │
│  - WAL Storage                      │
└─────────────────────────────────────┘
```

## 2. 架构设计

### 2.1 类图

```
┌─────────────────────────┐
│   StorageManager        │
│─────────────────────────│
│ - storage: Arc<dyn StorageInterface> │
│ - config: StorageConfig │
│ - metrics: Arc<StorageMetrics>       │
├─────────────────────────┤
│ + new() -> Self         │
│ + mount_index()         │
│ + sync_index()          │
│ + search()              │
│ + get_document()        │
│ + enrich_results()      │
│ + get_stats()           │
│ + health_check()        │
└─────────────────────────┘
            ▲
            │
            │
┌───────────┴─────────────────┐
│  MutableStorageManager      │
│─────────────────────────────│
│ - storage: Arc<RwLock<Box<dyn StorageInterface>>> │
├─────────────────────────────┤
│ + add_document()            │
│ + remove_document()         │
│ + batch_add()               │
│ + commit_changes()          │
│ + clear_all()               │
│ + backup()                  │
│ + restore()                 │
└─────────────────────────────┘
```

### 2.2 核心类型

#### 2.21 StorageManager（只读）

```rust
/// 只读存储管理器
/// 
/// 提供存储的只读操作，支持多线程共享
#[derive(Clone)]
pub struct StorageManager {
    /// 底层存储接口（Arc 包装，支持共享）
    storage: Arc<dyn StorageInterface>,
    
    /// 存储配置
    config: StorageConfig,
    
    /// 性能指标收集器
    metrics: Arc<StorageMetrics>,
}

impl StorageManager {
    /// 创建新的存储管理器
    pub fn new(
        storage: Arc<dyn StorageInterface>,
        config: StorageConfig
    ) -> Self {
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
}
```

**核心方法**:

```rust
impl StorageManager {
    // ========== 索引管理 ==========
    
    /// 挂载索引到存储
    /// 
    /// # Arguments
    /// * `index` - 要挂载的索引
    /// 
    /// # Returns
    /// * `Result<()>` - 操作结果
    /// 
    /// # Example
    /// ```rust
    /// let manager = StorageManager::new(storage, config);
    /// manager.mount_index(&index).await?;
    /// ```
    pub async fn mount_index(&self, index: &Index) -> Result<()> {
        let _timer = self.metrics.operation_timer("mount_index");
        
        // 1. 验证索引有效性
        self.validate_index(index)?;
        
        // 2. 序列化索引数据
        let data = self.serialize_index(index)?;
        
        // 3. 存储到后端
        self.storage.mount(index).await?;
        
        // 4. 记录元数据
        self.record_metadata(&data).await?;
        
        Ok(())
    }
    
    /// 同步索引到存储
    pub async fn sync_index(&self, index: &Index) -> Result<()> {
        let _timer = self.metrics.operation_timer("sync_index");
        
        // 使用增量提交减少 IO
        self.storage.commit(index, CommitMode::Incremental).await
    }
    
    // ========== 数据查询 ==========
    
    /// 搜索术语
    pub async fn search(
        &self,
        term: &str,
        options: SearchOptions
    ) -> Result<SearchResults> {
        let _timer = self.metrics.operation_timer("search");
        self.metrics.record_read();
        
        // 1. 尝试从缓存获取
        if let Some(cached) = self.get_from_cache(term).await {
            self.metrics.record_cache_hit();
            return Ok(cached);
        }
        
        // 2. 从存储查询
        let results = self.storage.get(term, None, options).await?;
        
        // 3. 异步缓存结果
        self.cache_result(term, &results).await;
        
        Ok(results)
    }
    
    /// 获取文档内容
    pub async fn get_document(&self, id: DocId) -> Result<Option<String>> {
        let _timer = self.metrics.operation_timer("get_document");
        self.metrics.record_read();
        
        self.storage.get_document(id).await
    }
    
    /// 批量获取文档
    pub async fn get_documents(&self, ids: &[DocId]) -> Result<Vec<Option<String>>> {
        let _timer = self.metrics.operation_timer("get_documents");
        
        // 并发获取
        let futures: Vec<_> = ids.iter()
            .map(|&id| self.storage.get_document(id))
            .collect();
        
        futures::future::try_join_all(futures).await
    }
    
    /// 富化搜索结果
    pub async fn enrich_results(
        &self,
        results: &SearchResults
    ) -> Result<EnrichedSearchResults> {
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
            read_count: self.metrics.read_count(),
            write_count: self.metrics.write_count(),
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
            details: self.collect_health_details().await,
        })
    }
}
```

#### 2.22 MutableStorageManager（可变）

```rust
/// 可变存储管理器
/// 
/// 支持写操作，使用 RwLock 保证线程安全
pub struct MutableStorageManager {
    /// 底层存储接口（RwLock 包装，支持可变操作）
    storage: Arc<RwLock<Box<dyn StorageInterface>>>,
    
    /// 存储配置
    config: StorageConfig,
    
    /// 性能指标收集器
    metrics: Arc<StorageMetrics>,
}

impl MutableStorageManager {
    /// 创建新的可变存储管理器
    pub fn new(
        storage: Box<dyn StorageInterface>,
        config: StorageConfig
    ) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            config,
            metrics: Arc::new(StorageMetrics::default()),
        }
    }
    
    /// 从 Arc 创建（用于兼容现有代码）
    pub fn from_arc(
        storage: Arc<dyn StorageInterface>,
        config: StorageConfig
    ) -> Self {
        Self {
            storage: Arc::new(RwLock::new(
                Box::new(ArcStorageWrapper(storage))
            )),
            config,
            metrics: Arc::new(StorageMetrics::default()),
        }
    }
}
```

**核心方法**:

```rust
impl MutableStorageManager {
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
        
        // 先删除再添加（事务性）
        storage.remove_document(id).await?;
        storage.store_document(id, content.to_string()).await?;
        
        Ok(())
    }
    
    // ========== 批量操作 ==========
    
    /// 批量添加文档
    pub async fn batch_add(
        &self,
        documents: &[(DocId, String)]
    ) -> Result<()> {
        let _timer = self.metrics.operation_timer("batch_add");
        
        // 转换为变更列表
        let changes: Vec<IndexChange> = documents
            .iter()
            .map(|(id, content)| IndexChange::Add {
                doc_id: *id,
                content: content.clone(),
            })
            .collect();
        
        // 批量提交
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
    pub async fn batch_update(
        &self,
        documents: &[(DocId, String)]
    ) -> Result<()> {
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
    pub async fn sync_with_index(&self, index: &Index) -> Result<()> {
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
```

## 3. 关键设计决策

### 3.1 为什么区分 StorageManager 和 MutableStorageManager？

**设计考虑**:

1. **读写分离**: 
   - 读操作远多于写操作
   - 读操作可以共享，写操作需要独占锁
   
2. **性能优化**:
   - `StorageManager` 使用 `Arc`，无锁读取
   - `MutableStorageManager` 使用 `RwLock`，写时加锁

3. **类型安全**:
   - 编译期区分读写能力
   - 避免意外的写操作

**使用场景**:

```rust
// 只读场景（如搜索服务）
fn create_search_service(storage: Arc<dyn StorageInterface>) {
    let manager = StorageManager::new(storage, config);
    // 只能执行读操作
}

// 读写场景（如完整服务）
fn create_full_service(storage: Box<dyn StorageInterface>) {
    let manager = MutableStorageManager::new(storage, config);
    // 可以执行读写操作
}
```

### 3.2 为什么使用 Arc<RwLock<Box<dyn StorageInterface>>>？

**各层包装的作用**:

1. **`dyn StorageInterface`**: 动态分发，支持不同存储实现
2. **`Box<>`**: 堆上分配，trait object 需要
3. **`RwLock<>`**: 线程安全的可变访问
4. **`Arc<>`**: 多线程共享所有权

**内存布局**:

```
MutableStorageManager
  │
  ├─> Arc (引用计数)
       │
       └─> RwLock (读写锁)
            │
            └─> Box (堆指针)
                 │
                 └─> dyn StorageInterface (vtable)
                      │
                      └─> ConcreteStorage (实际类型)
```

### 3.3 错误处理策略

```rust
pub enum StorageError {
    /// 连接错误（可恢复）
    ConnectionError(String),
    
    /// 序列化错误（通常不可恢复）
    SerializationError(String),
    
    /// 数据不一致（需要人工介入）
    DataInconsistencyError(String),
    
    /// 超时错误（可重试）
    TimeoutError(Duration),
    
    /// 存储已满（需要扩容）
    StorageFullError,
    
    /// 不支持的操作
    UnsupportedOperation(String),
    
    /// 内部错误
    InternalError(String),
}

impl StorageManager {
    /// 带重试的错误处理
    async fn with_retry<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        let mut attempts = 0;
        let max_attempts = self.config.max_retry_attempts;
        
        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                
                Err(Error::Storage(StorageError::TimeoutError(_))) 
                | Err(Error::Storage(StorageError::ConnectionError(_))) 
                if attempts < max_attempts => {
                    attempts += 1;
                    let delay = self.calculate_backoff(attempts);
                    tokio::time::sleep(delay).await;
                    continue;
                }
                
                Err(e) => {
                    self.metrics.record_error(&e);
                    return Err(e);
                }
            }
       }
    }
}
```

### 3.4 指标收集设计

```rust
pub struct StorageMetrics {
    // 操作计数
    reads: AtomicU64,
    writes: AtomicU64,
    deletes: AtomicU64,
    errors: AtomicU64,
    
    // 缓存统计
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    
    // 延迟统计（使用 HDR Histogram）
    read_latency: RwLock<Histogram>,
    write_latency: RwLock<Histogram>,
    
    // 存储大小
    storage_size_bytes: AtomicU64,
    document_count: AtomicU64,
}

impl StorageMetrics {
    /// 记录读操作
    pub fn record_read(&self) {
        self.reads.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录写操作
    pub fn record_write(&self) {
        self.writes.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存命中
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录缓存未命中
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    /// 记录错误
    pub fn record_error(&self, error: &Error) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        tracing::warn!("Storage error: {:?}", error);
    }
    
    /// 操作计时器
    pub fn operation_timer(&self, operation: &'static str) -> OperationTimer {
        OperationTimer::new(operation, self)
    }
    
    /// 导出 Prometheus 指标
    pub fn export_prometheus(&self) -> String {
        format!(
            r#"
            storage_reads_total {}
            storage_writes_total {}
            storage_cache_hits_total {}
            storage_cache_misses_total {}
            storage_errors_total {}
            "#,
            self.reads.load(Ordering::Relaxed),
            self.writes.load(Ordering::Relaxed),
            self.cache_hits.load(Ordering::Relaxed),
            self.cache_misses.load(Ordering::Relaxed),
            self.errors.load(Ordering::Relaxed),
        )
    }
}

/// 操作计时器（RAII 模式）
pub struct OperationTimer<'a> {
    operation: &'static str,
    metrics: &'a StorageMetrics,
    start: Instant,
}

impl<'a> OperationTimer<'a> {
    pub fn new(operation: &'static str, metrics: &'a StorageMetrics) -> Self {
        Self {
            operation,
            metrics,
            start: Instant::now(),
        }
    }
}

impl<'a> Drop for OperationTimer<'a> {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        let latency_ns = duration.as_nanos() as u64;
        
        // 记录到 histogram
        match self.operation {
            "read" | "search" | "get_document" => {
                self.metrics.read_latency.write().unwrap().record(latency_ns);
            }
            "write" | "add_document" | "commit" => {
                self.metrics.write_latency.write().unwrap().record(latency_ns);
            }
            _ => {}
        }
        
        // 记录慢查询
        if duration > Duration::from_millis(100) {
            tracing::warn!("Slow storage operation: {} took {:?}", 
                          self.operation, duration);
        }
    }
}
```

## 4. 使用示例

### 4.1 基本使用

```rust
use inversearch::storage::{StorageManager, MutableStorageManager, StorageFactory};

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 创建存储
    let config = StorageConfig::default();
    let storage = StorageFactory::create(&config).await?;
    
    // 2. 创建管理器
    let read_manager = StorageManager::new(storage.clone(), config.clone());
    let write_manager = MutableStorageManager::new(
        Box::new(storage),
        config.clone()
    );
    
    // 3. 写操作
    write_manager.add_document(1, "Hello World").await?;
    
    // 4. 读操作
    let doc = read_manager.get_document(1).await?;
    println!("Document: {:?}", doc);
    
    // 5. 健康检查
    let status = read_manager.health_check().await?;
    println!("Health: {:?}", status);
    
    Ok(())
}
```

### 4.2 与 Index 集成

```rust
use inversearch::{Index, IndexOptions};
use inversearch::storage::MutableStorageManager;

async fn create_index_with_storage() -> Result<Index> {
    // 1. 创建存储
    let storage = StorageFactory::create(&config).await?;
    
    // 2. 创建管理器
    let storage_manager = MutableStorageManager::new(
        Box::new(storage),
        config
    );
    
    // 3. 创建带存储的索引
    let index = Index::with_storage(
        IndexOptions::default(),
        storage_manager
    ).await?;
    
    Ok(index)
}

async fn use_index(index: &mut Index) -> Result<()> {
    // 添加文档（自动同步到存储）
    index.add_async(1, "Document content").await?;
    
    // 搜索（可能从缓存）
    let results = index.search_async("query", options).await?;
    
    Ok(())
}
```

### 4.3 批量操作

```rust
async fn batch_import(
    storage: &MutableStorageManager,
    documents: Vec<(DocId, String)>
) -> Result<()> {
    // 分批处理，避免一次性占用太多内存
    const BATCH_SIZE: usize = 1000;
    
    for batch in documents.chunks(BATCH_SIZE) {
        storage.batch_add(batch).await?;
        
        // 可选：每批之间短暂暂停，避免 IO 压力
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    Ok(())
}
```

## 5. 性能考虑

### 5.1 异步非阻塞

所有存储操作都是异步的，不阻塞主线程：

```rust
// 好的做法
index.add_async(id, content).await?;  // 异步

// 避免
index.add(id, content)?;  // 同步阻塞
```

### 5.2 批量优化

```rust
// 差：多次单独写入
for (id, content) in documents {
    storage.add_document(id, content).await?;
}

// 好：批量写入
storage.batch_add(&documents).await?;
```

### 5.3 缓存策略

```rust
// 配置缓存
let config = StorageConfig {
    cache: CacheConfig {
        enabled: true,
        hot_size: 1_000_000_000,  // 1GB
        warm_size: 4_000_000_000, // 4GB
        ttl: Duration::from_secs(3600),
    },
    ..Default::default()
};
```

## 6. 测试策略

### 6.1 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_storage_manager_creation() {
        let storage = MockStorage::new();
        let manager = StorageManager::new(storage, config);
        
        assert!(manager.health_check().await.unwrap().is_healthy);
    }
    
    #[tokio::test]
    async fn test_mutable_manager_write() {
        let storage = MockStorage::new();
        let manager = MutableStorageManager::new(Box::new(storage), config);
        
        manager.add_document(1, "test").await.unwrap();
        
        let doc = manager.get_document(1).await.unwrap();
        assert_eq!(doc, Some("test".to_string()));
    }
}
```

### 6.2 集成测试

```rust
#[cfg(test)]
mod integration {
    #[tokio::test]
    async fn test_end_to_end() {
        // 1. 创建临时存储
        let temp_dir = tempfile::tempdir().unwrap();
        let storage = FileStorage::new(temp_dir.path());
        
        // 2. 创建管理器
        let manager = MutableStorageManager::new(Box::new(storage), config);
        
        // 3. 完整流程测试
        manager.add_document(1, "doc1").await.unwrap();
        manager.add_document(2, "doc2").await.unwrap();
        
        let doc1 = manager.get_document(1).await.unwrap();
        assert_eq!(doc1, Some("doc1".to_string()));
        
        // 4. 重启验证（数据持久化）
        drop(manager);
        let storage2 = FileStorage::new(temp_dir.path());
        let manager2 = StorageManager::new(storage2, config);
        
        let doc1_again = manager2.get_document(1).await.unwrap();
        assert_eq!(doc1_again, Some("doc1".to_string()));
    }
}
```

## 7. 总结

StorageManager 设计要点：

1. ✅ **分层清晰**: Manager → Interface → Implementation
2. ✅ **读写分离**: StorageManager (读) + MutableStorageManager (写)
3. ✅ **线程安全**: Arc + RwLock 保证并发安全
4. ✅ **指标完善**: 完整的性能指标收集
5. ✅ **错误处理**: 分类明确的错误类型和重试机制
6. ✅ **易于测试**: 通过 trait 抽象支持 mock

通过 StorageManager，Inversearch 的存储模块实现了清晰的架构分层，为业务逻辑提供了统一、高效、可靠的存储抽象。
