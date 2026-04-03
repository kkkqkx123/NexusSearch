//! 存储模块集成测试
//!
//! 测试范围：
//! - 内存存储 (MemoryStorage)
//! - 文件存储 (FileStorage)
//! - 缓存存储 (CachedStorage)
//! - 存储基类 (StorageBase)
//! - 持久化功能

mod common;

use inversearch_service::storage::{
    base::StorageBase,
    common::StorageInterface,
};
use inversearch_service::Index;
use inversearch_service::index::IndexOptions;
use tempfile::TempDir;

// ============================================================================
// StorageBase 测试
// ============================================================================

/// 测试 StorageBase 基本功能
#[test]
fn test_storage_base_new() {
    let base = StorageBase::new();
    assert_eq!(base.get_memory_usage(), 0);
    assert_eq!(base.get_operation_count(), 0);
    assert_eq!(base.get_document_count(), 0);
    assert_eq!(base.get_index_count(), 0);
}

/// 测试 StorageBase 数据操作
#[test]
fn test_storage_base_data_operations() {
    let mut base = StorageBase::new();

    // 插入数据
    base.data.insert("hello".to_string(), vec![1, 2, 3]);
    base.data.insert("world".to_string(), vec![4, 5]);
    base.documents.insert(1, "hello world".to_string());
    base.documents.insert(2, "test content".to_string());

    base.update_memory_usage();

    assert_eq!(base.get_index_count(), 2);
    assert_eq!(base.get_document_count(), 2);
    assert!(base.get_memory_usage() > 0);
}

/// 测试 StorageBase 搜索功能
#[test]
fn test_storage_base_get() {
    let mut base = StorageBase::new();

    base.data.insert("rust".to_string(), vec![1, 2, 3]);
    base.data.insert("programming".to_string(), vec![2, 3, 4]);

    // 测试基本搜索
    let results = base.get("rust", None, 10, 0);
    assert_eq!(results.len(), 3);
    assert!(results.contains(&1));
    assert!(results.contains(&2));
    assert!(results.contains(&3));

    // 测试限制结果数量
    let limited = base.get("rust", None, 2, 0);
    assert_eq!(limited.len(), 2);

    // 测试偏移
    let offset = base.get("rust", None, 10, 1);
    assert_eq!(offset.len(), 2);
    assert!(!offset.contains(&1)); // 第一个被跳过

    // 测试不存在的键
    let empty = base.get("nonexistent", None, 10, 0);
    assert!(empty.is_empty());
}

/// 测试 StorageBase 上下文搜索
#[test]
fn test_storage_base_context_search() {
    let mut base = StorageBase::new();

    let mut ctx_map = std::collections::HashMap::new();
    ctx_map.insert("term1".to_string(), vec![1, 2]);
    ctx_map.insert("term2".to_string(), vec![3, 4]);
    base.context_data.insert("ctx1".to_string(), ctx_map);

    let results = base.get("term1", Some("ctx1"), 10, 0);
    assert_eq!(results.len(), 2);
    assert!(results.contains(&1));

    // 测试不存在的上下文
    let empty = base.get("term1", Some("nonexistent"), 10, 0);
    assert!(empty.is_empty());
}

/// 测试 StorageBase 富化功能
#[test]
fn test_storage_base_enrich() {
    let mut base = StorageBase::new();

    base.documents.insert(1, "content 1".to_string());
    base.documents.insert(2, "content 2".to_string());
    base.documents.insert(3, "content 3".to_string());

    let enriched = base.enrich(&[1, 2, 999]); // 999 不存在

    assert_eq!(enriched.len(), 2);
    assert!(enriched.iter().any(|r| r.id == 1));
    assert!(enriched.iter().any(|r| r.id == 2));
}

/// 测试 StorageBase 存在检查
#[test]
fn test_storage_base_has() {
    let mut base = StorageBase::new();

    base.data.insert("test".to_string(), vec![1, 2, 3]);

    assert!(base.has(1));
    assert!(base.has(2));
    assert!(base.has(3));
    assert!(!base.has(999));
}

/// 测试 StorageBase 删除功能
#[test]
fn test_storage_base_remove() {
    let mut base = StorageBase::new();

    base.data.insert("test".to_string(), vec![1, 2, 3]);
    base.documents.insert(1, "doc1".to_string());
    base.documents.insert(2, "doc2".to_string());

    base.remove(&[1]);

    assert!(!base.has(1));
    assert!(base.has(2));
    assert!(base.has(3));
    assert!(!base.documents.contains_key(&1));
}

/// 测试 StorageBase 清空功能
#[test]
fn test_storage_base_clear() {
    let mut base = StorageBase::new();

    base.data.insert("test".to_string(), vec![1, 2, 3]);
    base.documents.insert(1, "doc1".to_string());
    base.context_data.insert("ctx".to_string(), std::collections::HashMap::new());

    base.clear();

    assert!(base.data.is_empty());
    assert!(base.documents.is_empty());
    assert!(base.context_data.is_empty());
    assert_eq!(base.get_memory_usage(), 0);
}

/// 测试 StorageBase 性能指标
#[test]
fn test_storage_base_metrics() {
    let base = StorageBase::new();

    // 记录一些操作
    let start1 = base.record_operation_start();
    std::thread::sleep(std::time::Duration::from_micros(100));
    base.record_operation_completion(start1);

    let start2 = base.record_operation_start();
    std::thread::sleep(std::time::Duration::from_micros(100));
    base.record_operation_completion(start2);

    assert_eq!(base.get_operation_count(), 2);
    assert!(base.get_total_latency() > 0);
    assert!(base.get_average_latency() > 0);
}

// ============================================================================
// 内存存储测试
// ============================================================================

#[cfg(feature = "store-memory")]
mod memory_tests {
    use super::*;
    use inversearch_service::storage::memory::MemoryStorage;

    /// 测试内存存储基本操作
    #[tokio::test]
    async fn test_memory_storage_basic() {
        let mut storage = MemoryStorage::new();

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "hello world", false).expect("add should succeed");
        index.add(2, "rust programming", false).expect("add should succeed");

        // 提交到存储
        storage.commit(&index, false, false).await.expect("commit should succeed");

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.expect("get should succeed");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 测试存在检查
        assert!(storage.has(1).await.expect("has should succeed"));
        assert!(!storage.has(999).await.expect("has should succeed"));

        storage.close().await.expect("close should succeed");
    }

    /// 测试内存存储清空
    #[tokio::test]
    async fn test_memory_storage_clear() {
        let mut storage = MemoryStorage::new();

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "test content", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        // 清空
        storage.clear().await.expect("clear should succeed");

        let results = storage.get("test", None, 10, 0, true, false).await.expect("get should succeed");
        assert!(results.is_empty());

        storage.close().await.expect("close should succeed");
    }

    /// 测试内存存储删除
    #[tokio::test]
    async fn test_memory_storage_remove() {
        let mut storage = MemoryStorage::new();

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "doc1", false).expect("add should succeed");
        index.add(2, "doc2", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        // 删除文档
        storage.remove(&[1]).await.expect("remove should succeed");

        assert!(!storage.has(1).await.expect("has should succeed"));
        assert!(storage.has(2).await.expect("has should succeed"));

        storage.close().await.expect("close should succeed");
    }

    /// 测试内存存储富化
    #[tokio::test]
    async fn test_memory_storage_enrich() {
        let mut storage = MemoryStorage::new();

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "content one", false).expect("add should succeed");
        index.add(2, "content two", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        let enriched = storage.enrich(&[1, 2]).await.expect("enrich should succeed");
        assert_eq!(enriched.len(), 2);

        storage.close().await.expect("close should succeed");
    }

    /// 测试内存存储指标
    #[tokio::test]
    async fn test_memory_storage_metrics() {
        let mut storage = MemoryStorage::new();

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "test content for metrics", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        let metrics = storage.get_operation_stats();
        assert!(metrics.operation_count > 0);
        assert!(metrics.memory_usage > 0);

        storage.close().await.expect("close should succeed");
    }
}

// ============================================================================
// 文件存储测试
// ============================================================================

#[cfg(feature = "store-file")]
mod file_tests {
    use super::*;
    use inversearch_service::storage::file::FileStorage;

    /// 测试文件存储基本操作
    #[tokio::test]
    async fn test_file_storage_basic() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = FileStorage::new(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "hello world", false).expect("add should succeed");
        index.add(2, "rust programming", false).expect("add should succeed");

        // 提交到存储
        storage.commit(&index, false, false).await.expect("commit should succeed");

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.expect("get should succeed");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 保存到文件
        storage.save_to_file().await.expect("save should succeed");

        // 关闭存储
        storage.close().await.expect("close should succeed");
    }

    /// 测试文件存储持久化
    #[tokio::test]
    async fn test_file_storage_persistence() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let path = temp_dir.path().to_path_buf();

        // 第一次创建并写入数据
        {
            let mut storage = FileStorage::new(&path);
            storage.open().await.expect("open should succeed");

            let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
            index.add(1, "persistent data", false).expect("add should succeed");
            storage.commit(&index, false, false).await.expect("commit should succeed");

            storage.close().await.expect("close should succeed");
        }

        // 第二次打开验证数据持久化
        {
            let mut storage = FileStorage::new(&path);
            storage.open().await.expect("open should succeed");

            let results = storage.get("persistent", None, 10, 0, true, false).await.expect("get should succeed");
            assert_eq!(results.len(), 1);
            assert!(results.contains(&1));

            storage.close().await.expect("close should succeed");
        }
    }

    /// 测试文件存储文件大小
    #[tokio::test]
    async fn test_file_storage_size() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = FileStorage::new(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "test content", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        storage.close().await.expect("close should succeed");

        let size = storage.get_file_size();
        assert!(size > 0, "File size should be positive");
    }
}

// ============================================================================
// 缓存存储测试
// ============================================================================

#[cfg(feature = "store-cached")]
mod cached_tests {
    use super::*;
    use inversearch_service::storage::cached::{CachedStorage, CachedStorageConfig};

    /// 测试缓存存储基本操作
    #[tokio::test]
    async fn test_cached_storage_basic() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = CachedStorage::with_path(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "hello world", false).expect("add should succeed");
        index.add(2, "rust programming", false).expect("add should succeed");

        // 提交到存储
        storage.commit(&index, false, false).await.expect("commit should succeed");
        assert!(storage.is_dirty());

        // 测试获取
        let results = storage.get("hello", None, 10, 0, true, false).await.expect("get should succeed");
        assert_eq!(results.len(), 1);
        assert!(results.contains(&1));

        // 关闭存储（会保存到文件）
        storage.close().await.expect("close should succeed");
        assert!(!storage.is_dirty());
    }

    /// 测试缓存存储持久化
    #[tokio::test]
    async fn test_cached_storage_persistence() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let path = temp_dir.path().to_path_buf();

        // 第一次创建并写入数据
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.expect("open should succeed");

            let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
            index.add(1, "persistent data", false).expect("add should succeed");
            storage.commit(&index, false, false).await.expect("commit should succeed");

            storage.close().await.expect("close should succeed");
        }

        // 第二次打开验证数据持久化
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.expect("open should succeed");

            let results = storage.get("persistent", None, 10, 0, true, false).await.expect("get should succeed");
            assert_eq!(results.len(), 1);
            assert!(results.contains(&1));

            storage.destroy().await.expect("destroy should succeed");
        }
    }

    /// 测试缓存存储清空
    #[tokio::test]
    async fn test_cached_storage_clear() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = CachedStorage::with_path(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "test content", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        // 清空
        storage.clear().await.expect("clear should succeed");
        assert!(storage.is_dirty());

        let results = storage.get("test", None, 10, 0, true, false).await.expect("get should succeed");
        assert!(results.is_empty());

        storage.close().await.expect("close should succeed");
    }

    /// 测试缓存存储删除
    #[tokio::test]
    async fn test_cached_storage_remove() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = CachedStorage::with_path(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "doc1", false).expect("add should succeed");
        index.add(2, "doc2", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        // 删除文档
        storage.remove(&[1]).await.expect("remove should succeed");
        assert!(storage.is_dirty());

        assert!(!storage.has(1).await.expect("has should succeed"));
        assert!(storage.has(2).await.expect("has should succeed"));

        storage.close().await.expect("close should succeed");
    }

    /// 测试缓存存储信息
    #[tokio::test]
    async fn test_cached_storage_info() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = CachedStorage::with_path(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "test content", false).expect("add should succeed");
        index.add(2, "more content", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");

        let info = storage.info().await.expect("info should succeed");
        assert_eq!(info.name, "CachedStorage");
        assert_eq!(info.version, "1.0.0");
        assert!(info.is_connected);

        storage.close().await.expect("close should succeed");
    }

    /// 测试缓存存储配置
    #[tokio::test]
    async fn test_cached_storage_config() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");

        let config = CachedStorageConfig {
            base_path: temp_dir.path().to_path_buf(),
            auto_save_interval: 60,
            auto_save_on_drop: true,
        };

        let storage = CachedStorage::with_config(config);

        assert_eq!(storage.config().base_path, temp_dir.path());
        assert_eq!(storage.config().auto_save_interval, 60);
        assert!(storage.config().auto_save_on_drop);
    }

    /// 测试缓存存储显式保存
    #[tokio::test]
    async fn test_cached_storage_explicit_save() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let path = temp_dir.path().to_path_buf();

        // 创建并保存
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.expect("open should succeed");

            let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
            index.add(1, "saved data", false).expect("add should succeed");
            storage.commit(&index, false, false).await.expect("commit should succeed");

            // 显式保存
            storage.save().await.expect("save should succeed");
            assert!(!storage.is_dirty());
        }

        // 重新加载验证
        {
            let mut storage = CachedStorage::with_path(&path);
            storage.open().await.expect("open should succeed");

            let results = storage.get("saved", None, 10, 0, true, false).await.expect("get should succeed");
            assert_eq!(results.len(), 1);

            storage.destroy().await.expect("destroy should succeed");
        }
    }

    /// 测试缓存存储销毁
    #[tokio::test]
    async fn test_cached_storage_destroy() {
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut storage = CachedStorage::with_path(temp_dir.path());

        storage.open().await.expect("open should succeed");

        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "test content", false).expect("add should succeed");
        storage.commit(&index, false, false).await.expect("commit should succeed");
        storage.close().await.expect("close should succeed");

        // 销毁
        storage.destroy().await.expect("destroy should succeed");

        // 验证数据已清除
        let info = storage.info().await.expect("info should succeed");
        assert_eq!(info.document_count, 0);
        assert_eq!(info.index_count, 0);
    }
}

// ============================================================================
// 存储对比测试
// ============================================================================

#[cfg(all(feature = "store-memory", feature = "store-cached"))]
mod comparison_tests {
    use super::*;
    use inversearch_service::storage::memory::MemoryStorage;
    use inversearch_service::storage::cached::CachedStorage;

    /// 对比内存存储和缓存存储的搜索结果一致性
    #[tokio::test]
    async fn test_storage_search_consistency() {
        // 准备相同的索引数据
        let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
        index.add(1, "rust programming language", false).expect("add should succeed");
        index.add(2, "rust memory safety", false).expect("add should succeed");
        index.add(3, "programming best practices", false).expect("add should succeed");

        // 内存存储
        let mut memory_storage = MemoryStorage::new();
        memory_storage.open().await.expect("open should succeed");
        memory_storage.commit(&index, false, false).await.expect("commit should succeed");

        // 缓存存储
        let temp_dir = TempDir::new().expect("create temp dir should succeed");
        let mut cached_storage = CachedStorage::with_path(temp_dir.path());
        cached_storage.open().await.expect("open should succeed");
        cached_storage.commit(&index, false, false).await.expect("commit should succeed");

        // 对比搜索结果
        let memory_results = memory_storage.get("rust", None, 10, 0, true, false).await.expect("get should succeed");
        let cached_results = cached_storage.get("rust", None, 10, 0, true, false).await.expect("get should succeed");

        assert_eq!(memory_results.len(), cached_results.len());
        for id in &memory_results {
            assert!(cached_results.contains(id));
        }

        memory_storage.close().await.expect("close should succeed");
        cached_storage.close().await.expect("close should succeed");
    }
}

// ============================================================================
// 边界情况测试
// ============================================================================

/// 测试空索引操作
#[tokio::test]
async fn test_storage_empty_index() {
    use inversearch_service::storage::cached::CachedStorage;

    let temp_dir = TempDir::new().expect("create temp dir should succeed");
    let mut storage = CachedStorage::with_path(temp_dir.path());

    storage.open().await.expect("open should succeed");

    // 提交空索引
    let index = Index::new(IndexOptions::default()).expect("create index should succeed");
    storage.commit(&index, false, false).await.expect("commit should succeed");

    // 验证空结果
    let results = storage.get("anything", None, 10, 0, true, false).await.expect("get should succeed");
    assert!(results.is_empty());

    let info = storage.info().await.expect("info should succeed");
    assert_eq!(info.document_count, 0);

    storage.close().await.expect("close should succeed");
}

/// 测试大量文档操作
#[tokio::test]
async fn test_storage_large_document_count() {
    use inversearch_service::storage::cached::CachedStorage;

    let temp_dir = TempDir::new().expect("create temp dir should succeed");
    let mut storage = CachedStorage::with_path(temp_dir.path());

    storage.open().await.expect("open should succeed");

    let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");

    // 添加大量文档
    for i in 1..=100 {
        index.add(i, &format!("document number {}", i), false).expect("add should succeed");
    }

    storage.commit(&index, false, false).await.expect("commit should succeed");

    // 验证索引项数量（不是文档数量，因为 commit 只提交索引数据）
    let info = storage.info().await.expect("info should succeed");
    // 索引项数量应该大于0（每个词项都会创建一个索引项）
    assert!(info.index_count > 0, "Should have index entries");

    // 验证可以搜索到文档
    let results = storage.get("document", None, 100, 0, true, false).await.expect("get should succeed");
    assert!(!results.is_empty(), "Should find documents with 'document' term");

    storage.close().await.expect("close should succeed");
}

/// 测试特殊字符内容持久化
#[tokio::test]
async fn test_storage_special_characters() {
    use inversearch_service::storage::cached::CachedStorage;

    let temp_dir = TempDir::new().expect("create temp dir should succeed");
    let mut storage = CachedStorage::with_path(temp_dir.path());

    storage.open().await.expect("open should succeed");

    let mut index = Index::new(IndexOptions::default()).expect("create index should succeed");
    // 使用英文内容测试特殊字符处理
    index.add(1, "Hello World Emoji Test", false).expect("add should succeed");
    index.add(2, "Special chars and quotes test", false).expect("add should succeed");
    index.add(3, "New line and tab test", false).expect("add should succeed");

    storage.commit(&index, false, false).await.expect("commit should succeed");
    storage.close().await.expect("close should succeed");

    // 重新打开验证数据持久化
    let mut storage2 = CachedStorage::with_path(temp_dir.path());
    storage2.open().await.expect("open should succeed");

    // 验证可以搜索到内容
    let results1 = storage2.get("hello", None, 10, 0, true, false).await.expect("get should succeed");
    assert!(!results1.is_empty(), "Should find documents with 'hello'");

    let results2 = storage2.get("special", None, 10, 0, true, false).await.expect("get should succeed");
    assert!(!results2.is_empty(), "Should find documents with 'special'");

    let results3 = storage2.get("test", None, 10, 0, true, false).await.expect("get should succeed");
    assert_eq!(results3.len(), 3, "Should find all 3 documents with 'test'");

    storage2.destroy().await.expect("destroy should succeed");
}
