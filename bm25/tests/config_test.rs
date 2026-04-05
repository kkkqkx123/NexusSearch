//! 配置模块测试
//!
//! 测试配置加载、验证和构建功能

use bm25_service::config::{Bm25Config, IndexManagerConfig};
use bm25_service::api::core::{MergePolicyType, IndexManager};
use tempfile::TempDir;

/// 测试 Bm25Config 默认值
///
/// 验证默认配置值符合预期
#[test]
fn test_bm25_config_defaults() {
    let config = Bm25Config::default();

    assert_eq!(config.k1, 1.2);
    assert_eq!(config.b, 0.75);
    assert_eq!(config.avg_doc_length, 100.0);
}

/// 测试 Bm25Config 构建器
///
/// 验证配置构建器正确设置值
#[test]
fn test_bm25_config_builder() {
    let config = Bm25Config::builder()
        .k1(1.5)
        .b(0.8)
        .avg_doc_length(200.0)
        .build();

    assert_eq!(config.k1, 1.5);
    assert_eq!(config.b, 0.8);
    assert_eq!(config.avg_doc_length, 200.0);
}

/// 测试 IndexManagerConfig 默认值
///
/// 验证索引管理器配置的默认值
#[test]
fn test_index_manager_config_defaults() {
    let config = IndexManagerConfig::default();

    assert!(config.reader_cache_enabled);
}

/// 测试 IndexManagerConfig 构建器
///
/// 验证索引管理器配置构建器正确设置值
#[test]
fn test_index_manager_config_builder() {
    let config = IndexManagerConfig::builder()
        .writer_memory_mb(100)
        .writer_threads(4)
        .reader_cache(false)
        .build();

    assert_eq!(config.writer_memory_budget, 100_000_000);
    assert_eq!(config.writer_num_threads, Some(4));
    assert!(!config.reader_cache_enabled);
}

/// 测试 IndexManagerConfig 创建索引
///
/// 验证使用自定义配置创建索引
#[test]
fn test_create_index_with_custom_config() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = IndexManagerConfig::builder()
        .writer_memory_mb(50)
        .writer_threads(2)
        .reader_cache(false)
        .build();

    let result = IndexManager::create_with_config(&index_path, config);
    assert!(result.is_ok(), "Should create index with custom config");
}

/// 测试 IndexManagerConfig 禁用缓存
///
/// 验证禁用 reader 缓存时的行为
#[test]
fn test_index_manager_config_no_cache() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = IndexManagerConfig::builder()
        .reader_cache(false)
        .build();

    let _manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");

    assert!(index_path.exists());
}

/// 测试 MergePolicyType 默认值
///
/// 验证合并策略类型的默认值
#[test]
fn test_merge_policy_type_default() {
    let default_policy = MergePolicyType::default();
    assert!(matches!(default_policy, MergePolicyType::Log));
}

/// 测试不同写入器内存配置
///
/// 验证不同的写入器内存配置
#[test]
fn test_different_writer_memory_configs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let memory_sizes = vec![16, 32, 64, 128, 256];

    for (i, memory_mb) in memory_sizes.iter().enumerate() {
        let index_path = temp_dir.path().join(format!("index_{}", i));
        let config = IndexManagerConfig::builder()
            .writer_memory_mb(*memory_mb)
            .build();

        let result = IndexManager::create_with_config(&index_path, config);
        assert!(result.is_ok(), "Should create index with {} MB memory", memory_mb);
    }
}

/// 测试不同写入器线程配置
///
/// 验证不同的写入器线程配置
#[test]
fn test_different_writer_thread_configs() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let thread_counts = vec![1, 2, 4, 8];

    for (i, threads) in thread_counts.iter().enumerate() {
        let index_path = temp_dir.path().join(format!("index_{}", i));
        let config = IndexManagerConfig::builder()
            .writer_threads(*threads)
            .build();

        let result = IndexManager::create_with_config(&index_path, config);
        assert!(result.is_ok(), "Should create index with {} threads", threads);
    }
}

/// 测试配置克隆
///
/// 验证配置可以正确克隆
#[test]
fn test_config_clone() {
    let config = IndexManagerConfig::builder()
        .writer_memory_mb(100)
        .writer_threads(4)
        .reader_cache(false)
        .build();

    let cloned = config.clone();

    assert_eq!(config.writer_memory_budget, cloned.writer_memory_budget);
    assert_eq!(config.writer_num_threads, cloned.writer_num_threads);
    assert_eq!(config.reader_cache_enabled, cloned.reader_cache_enabled);
}

/// 测试 Bm25Config 克隆
///
/// 验证 Bm25Config 可以正确克隆
#[test]
fn test_bm25_config_clone() {
    let config = Bm25Config::builder()
        .k1(1.5)
        .b(0.8)
        .avg_doc_length(200.0)
        .build();

    let cloned = config.clone();

    assert_eq!(config.k1, cloned.k1);
    assert_eq!(config.b, cloned.b);
    assert_eq!(config.avg_doc_length, cloned.avg_doc_length);
}

/// 测试配置 Debug 输出
///
/// 验证配置可以正确格式化输出
#[test]
fn test_config_debug_output() {
    let config = IndexManagerConfig::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("writer_memory_budget"));
    assert!(debug_str.contains("writer_num_threads"));
    assert!(debug_str.contains("reader_cache_enabled"));
}

/// 测试 Bm25Config Debug 输出
///
/// 验证 Bm25Config 可以正确格式化输出
#[test]
fn test_bm25_config_debug_output() {
    let config = Bm25Config::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("k1"));
    assert!(debug_str.contains("b"));
    assert!(debug_str.contains("avg_doc_length"));
}

/// 测试配置序列化
///
/// 验证配置可以正确序列化
#[test]
fn test_config_serialization() {
    let config = Bm25Config::builder()
        .k1(1.5)
        .b(0.8)
        .avg_doc_length(200.0)
        .build();

    let json = serde_json::to_string(&config);
    assert!(json.is_ok(), "Should serialize config to JSON");

    let json_str = json.unwrap();
    assert!(json_str.contains("k1"));
    assert!(json_str.contains("b"));
}

/// 测试配置反序列化
///
/// 验证配置可以正确反序列化
#[test]
fn test_config_deserialization() {
    let json = r#"{"k1":1.5,"b":0.8,"avg_doc_length":200.0,"field_weights":{"title":1.0,"content":1.0}}"#;

    let config: Result<Bm25Config, _> = serde_json::from_str(json);
    assert!(config.is_ok(), "Should deserialize config from JSON");

    let config = config.unwrap();
    assert_eq!(config.k1, 1.5);
    assert_eq!(config.b, 0.8);
    assert_eq!(config.avg_doc_length, 200.0);
}

/// 测试配置边界值 - k1 最小值
///
/// 验证 k1 参数接近最小值时的行为
#[test]
fn test_bm25_config_k1_minimum() {
    let config = Bm25Config::builder()
        .k1(0.0)
        .build();

    assert_eq!(config.k1, 0.0);
}

/// 测试配置边界值 - k1 最大值
///
/// 验证 k1 参数较大值时的行为
#[test]
fn test_bm25_config_k1_large() {
    let config = Bm25Config::builder()
        .k1(10.0)
        .build();

    assert_eq!(config.k1, 10.0);
}

/// 测试配置边界值 - b 最小值
///
/// 验证 b 参数接近最小值时的行为
#[test]
fn test_bm25_config_b_minimum() {
    let config = Bm25Config::builder()
        .b(0.0)
        .build();

    assert_eq!(config.b, 0.0);
}

/// 测试配置边界值 - b 最大值
///
/// 验证 b 参数接近最大值时的行为
#[test]
fn test_bm25_config_b_maximum() {
    let config = Bm25Config::builder()
        .b(1.0)
        .build();

    assert_eq!(config.b, 1.0);
}

/// 测试配置边界值 - 平均文档长度
///
/// 验证平均文档长度参数的行为
#[test]
fn test_bm25_config_avg_doc_length() {
    let config = Bm25Config::builder()
        .avg_doc_length(1.0)
        .build();

    assert_eq!(config.avg_doc_length, 1.0);

    let config = Bm25Config::builder()
        .avg_doc_length(10000.0)
        .build();

    assert_eq!(config.avg_doc_length, 10000.0);
}

/// 测试完整配置创建索引流程
///
/// 验证使用完整配置创建索引并进行操作
#[test]
fn test_full_config_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let _bm25_config = Bm25Config::builder()
        .k1(1.5)
        .b(0.75)
        .avg_doc_length(150.0)
        .build();

    let manager_config = IndexManagerConfig::builder()
        .writer_memory_mb(64)
        .writer_threads(2)
        .reader_cache(false)
        .build();

    let _manager = IndexManager::create_with_config(&index_path, manager_config)
        .expect("Failed to create index manager");

    assert!(index_path.exists());
}
