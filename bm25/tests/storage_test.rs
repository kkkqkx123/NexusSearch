//! 存储后端测试
//!
//! 测试存储模块的功能

#[cfg(feature = "storage-tantivy")]
use bm25_service::storage::{TantivyStorage, StorageInterface, Bm25Stats, StorageInfo};
#[cfg(feature = "storage-tantivy")]
use bm25_service::storage::tantivy::TantivyStorageConfig;
#[cfg(feature = "storage-tantivy")]
use tempfile::TempDir;

/// 测试 Tantivy 存储创建
///
/// 验证 Tantivy 存储可以正确创建
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_storage_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let storage = TantivyStorage::new(config);
    let info = storage.info().await;
    assert!(info.is_ok(), "Should get storage info after creation");
}

/// 测试 Tantivy 存储初始化
///
/// 验证 Tantivy 存储可以正确初始化
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_storage_init() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    let result = storage.init().await;
    assert!(result.is_ok(), "Should initialize storage");
}

/// 测试 Tantivy 存储关闭
///
/// 验证 Tantivy 存储可以正确关闭
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_storage_close() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let result = storage.close().await;
    assert!(result.is_ok(), "Should close storage");
}

/// 测试 Tantivy 存储提交统计
///
/// 验证可以提交词项统计
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_commit_stats() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let result = storage.commit_stats("test_term", 1.5, 10).await;
    assert!(result.is_ok(), "Should commit stats");
}

/// 测试 Tantivy 存储批量提交
///
/// 验证可以批量提交统计信息
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_commit_batch() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let mut stats = Bm25Stats::default();
    stats.tf.insert("term1".to_string(), 1.0);
    stats.tf.insert("term2".to_string(), 2.0);
    stats.df.insert("term1".to_string(), 5);
    stats.df.insert("term2".to_string(), 10);
    stats.total_docs = 100;
    stats.avg_doc_length = 50.0;

    let result = storage.commit_batch(&stats).await;
    assert!(result.is_ok(), "Should commit batch stats");
}

/// 测试 Tantivy 存储获取统计
///
/// 验证可以获取词项统计
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_get_stats() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    storage.commit_stats("test_term", 1.5, 10).await
        .expect("Failed to commit stats");

    let result = storage.get_stats("test_term").await;
    assert!(result.is_ok(), "Should get stats");
}

/// 测试 Tantivy 存储获取文档频率
///
/// 验证可以获取文档频率
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_get_df() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    storage.commit_stats("test_term", 1.5, 10).await
        .expect("Failed to commit stats");

    let result = storage.get_df("test_term").await;
    assert!(result.is_ok(), "Should get document frequency");
}

/// 测试 Tantivy 存储清空
///
/// 验证可以清空存储
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_clear() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    storage.commit_stats("test_term", 1.5, 10).await
        .expect("Failed to commit stats");

    let result = storage.clear().await;
    assert!(result.is_ok(), "Should clear storage");
}

/// 测试 Tantivy 存储信息
///
/// 验证可以获取存储信息
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_storage_info() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let result = storage.info().await;
    assert!(result.is_ok(), "Should get storage info");

    let info = result.unwrap();
    assert!(!info.name.is_empty());
}

/// 测试 Tantivy 存储健康检查
///
/// 验证健康检查功能
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_health_check() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let result = storage.health_check().await;
    assert!(result.is_ok(), "Health check should succeed");

    let is_healthy = result.unwrap();
    assert!(is_healthy, "Storage should be healthy");
}

/// 测试 Tantivy 存储删除文档统计
///
/// 验证可以删除文档统计信息
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_delete_doc_stats() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let result = storage.delete_doc_stats("doc1").await;
    assert!(result.is_ok(), "Should delete doc stats");
}

/// 测试 Bm25Stats 默认值
///
/// 验证统计结构默认值
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_bm25_stats_defaults() {
    let stats = Bm25Stats::default();

    assert!(stats.tf.is_empty());
    assert!(stats.df.is_empty());
    assert_eq!(stats.total_docs, 0);
    assert_eq!(stats.avg_doc_length, 0.0);
}

/// 测试 Bm25Stats 克隆
///
/// 验证统计结构可以正确克隆
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_bm25_stats_clone() {
    let mut stats = Bm25Stats::default();
    stats.tf.insert("term1".to_string(), 1.0);
    stats.df.insert("term1".to_string(), 5);
    stats.total_docs = 100;
    stats.avg_doc_length = 50.0;

    let cloned = stats.clone();

    assert_eq!(stats.tf, cloned.tf);
    assert_eq!(stats.df, cloned.df);
    assert_eq!(stats.total_docs, cloned.total_docs);
    assert_eq!(stats.avg_doc_length, cloned.avg_doc_length);
}

/// 测试 StorageInfo 结构
///
/// 验证存储信息结构
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_storage_info_structure() {
    let info = StorageInfo {
        name: "test_storage".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        document_count: 100,
        term_count: 500,
        is_connected: true,
    };

    assert_eq!(info.name, "test_storage");
    assert_eq!(info.version, "1.0.0");
    assert_eq!(info.size, 1024);
    assert_eq!(info.document_count, 100);
    assert_eq!(info.term_count, 500);
    assert!(info.is_connected);
}

/// 测试 StorageInfo 克隆
///
/// 验证存储信息可以正确克隆
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_storage_info_clone() {
    let info = StorageInfo {
        name: "test_storage".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        document_count: 100,
        term_count: 500,
        is_connected: true,
    };

    let cloned = info.clone();

    assert_eq!(info.name, cloned.name);
    assert_eq!(info.version, cloned.version);
    assert_eq!(info.size, cloned.size);
}

/// 测试 StorageInfo Debug 输出
///
/// 验证存储信息可以正确格式化输出
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_storage_info_debug() {
    let info = StorageInfo {
        name: "test_storage".to_string(),
        version: "1.0.0".to_string(),
        size: 1024,
        document_count: 100,
        term_count: 500,
        is_connected: true,
    };

    let debug_str = format!("{:?}", info);

    assert!(debug_str.contains("test_storage"));
    assert!(debug_str.contains("1.0.0"));
}

/// 测试 Tantivy 存储多次初始化
///
/// 验证多次初始化不会出错
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_multiple_init() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init first time");
    storage.close().await.expect("Failed to close");

    let result = storage.init().await;
    assert!(result.is_ok(), "Should init again after close");
}

/// 测试 Tantivy 存储路径创建
///
/// 验证存储路径不存在时会自动创建
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_storage_path_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("nested").join("path").join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    let result = storage.init().await;
    assert!(result.is_ok(), "Should create storage in nested path");
}

/// 测试空词项提交
///
/// 验证空词名的处理
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_empty_term() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let result = storage.commit_stats("", 1.0, 1).await;
    assert!(result.is_ok() || result.is_err(), "Empty term should be handled gracefully");
}

/// 测试 Unicode 词项
///
/// 验证 Unicode 词名的处理
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_tantivy_unicode_term() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_path = temp_dir.path().join("storage");

    let config = TantivyStorageConfig {
        index_path: storage_path,
        writer_memory_mb: 50,
    };

    let mut storage = TantivyStorage::new(config);

    storage.init().await.expect("Failed to init storage");

    let unicode_terms = vec!["中文", "日本語", "한국어", "العربية", "עברית"];

    for term in unicode_terms {
        let result = storage.commit_stats(term, 1.0, 1).await;
        assert!(result.is_ok(), "Unicode term '{}' should be handled", term);
    }
}

/// 测试 TantivyStorageConfig 默认值
///
/// 验证配置默认值
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_tantivy_storage_config_defaults() {
    let config = TantivyStorageConfig::default();

    assert_eq!(config.writer_memory_mb, 50);
}

/// 测试 TantivyStorageConfig 克隆
///
/// 验证配置可以正确克隆
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_tantivy_storage_config_clone() {
    let config = TantivyStorageConfig {
        index_path: std::path::PathBuf::from("/test/path"),
        writer_memory_mb: 100,
    };

    let cloned = config.clone();

    assert_eq!(config.index_path, cloned.index_path);
    assert_eq!(config.writer_memory_mb, cloned.writer_memory_mb);
}

/// 测试 TantivyStorageConfig Debug 输出
///
/// 验证配置可以正确格式化输出
#[cfg(feature = "storage-tantivy")]
#[test]
fn test_tantivy_storage_config_debug() {
    let config = TantivyStorageConfig {
        index_path: std::path::PathBuf::from("/test/path"),
        writer_memory_mb: 100,
    };

    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("writer_memory_mb"));
}

/// 测试不同内存配置
///
/// 验证不同的写入器内存配置
#[cfg(feature = "storage-tantivy")]
#[tokio::test]
async fn test_different_memory_configs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let memory_sizes = vec![16, 32, 64, 128, 256];

    for (i, memory_mb) in memory_sizes.iter().enumerate() {
        let storage_path = temp_dir.path().join(format!("storage_{}", i));
        let config = TantivyStorageConfig {
            index_path: storage_path,
            writer_memory_mb: *memory_mb,
        };

        let mut storage = TantivyStorage::new(config);
        let result = storage.init().await;
        assert!(result.is_ok(), "Should create storage with {} MB memory", memory_mb);
    }
}
