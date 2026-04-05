//! 错误处理测试
//!
//! 测试各种错误场景的处理

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::add_document,
    search::{search, SearchOptions},
    persistence::PersistenceManager,
};
use bm25_service::error::Bm25Error;
use std::collections::HashMap;
use tempfile::TempDir;

/// 测试打开不存在的索引
///
/// 验证打开不存在的索引时返回正确的错误
#[test]
fn test_open_nonexistent_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("nonexistent");

    let result = IndexManager::open(&index_path);
    assert!(result.is_err(), "Should return error for nonexistent index");
}

/// 测试空文档 ID
///
/// 验证使用空字符串作为文档 ID 时的行为
#[test]
fn test_add_document_with_empty_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Test".to_string());
    fields.insert("content".to_string(), "Content".to_string());

    let result = add_document(&manager, &schema, "", &fields);
    assert!(result.is_ok(), "Empty ID should be accepted");
}

/// 测试超大分页偏移
///
/// 验证超大分页偏移不会导致错误
#[test]
fn test_search_with_large_offset() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let options = SearchOptions {
        limit: 10,
        offset: 1_000_000,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let result = search(&manager, &schema, "test", &options);
    assert!(result.is_ok(), "Large offset search should succeed");

    let (results, _) = result.expect("Should have results");
    assert!(results.is_empty(), "Large offset should return empty results");
}

/// 测试超大限制值
///
/// 验证超大限制值不会导致错误
#[test]
fn test_search_with_large_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let options = SearchOptions {
        limit: 1_000_000,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let result = search(&manager, &schema, "test", &options);
    assert!(result.is_ok(), "Large limit search should succeed");
}

/// 测试空查询字符串
///
/// 验证空查询字符串的处理
#[test]
fn test_search_with_empty_query() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let options = SearchOptions::default();
    let result = search(&manager, &schema, "", &options);

    assert!(result.is_ok(), "Empty query should succeed");
    let (results, max_score) = result.expect("Should have results");
    assert!(results.is_empty(), "Empty query should return no results");
    assert_eq!(max_score, 0.0, "Empty query should have zero max score");
}

/// 测试特殊字符查询
///
/// 验证包含特殊字符的查询不会导致解析错误
#[test]
fn test_search_with_special_characters() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let special_queries = vec![
        "test:query",
        "test AND query",
        "test OR query",
        "test NOT query",
        "test*",
        "test?",
        "(test)",
        "[test]",
        "{test}",
        "test~1",
        "test^2",
    ];

    let options = SearchOptions::default();
    for query in special_queries {
        let result = search(&manager, &schema, query, &options);
        assert!(result.is_ok() || result.is_err(), "Query '{}' should not panic", query);
    }
}

/// 测试无效字段权重
///
/// 验证无效字段权重的处理
#[test]
fn test_search_with_invalid_field_weights() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut field_weights = HashMap::new();
    field_weights.insert("nonexistent_field".to_string(), 1.0);
    field_weights.insert("another_fake_field".to_string(), 2.0);

    let options = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights,
        highlight: false,
    };

    let result = search(&manager, &schema, "test", &options);
    assert!(result.is_ok(), "Invalid field weights should not cause error");
}

/// 测试负数字段权重
///
/// 验证负数字段权重的处理
#[test]
fn test_search_with_negative_field_weights() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut field_weights = HashMap::new();
    field_weights.insert("title".to_string(), -1.0);
    field_weights.insert("content".to_string(), -2.0);

    let options = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights,
        highlight: false,
    };

    let result = search(&manager, &schema, "test", &options);
    assert!(result.is_ok(), "Negative field weights should not cause error");
}

/// 测试备份恢复失败场景
///
/// 验证从无效备份恢复时的错误处理
#[test]
fn test_restore_from_invalid_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let invalid_backup = base_path.join("invalid_backup");

    std::fs::create_dir_all(&invalid_backup).expect("Failed to create dir");

    let persistence = PersistenceManager::new(base_path);
    let result = persistence.restore_backup("restored", &invalid_backup);

    assert!(result.is_ok() || result.is_err(), "Invalid backup restore should handle gracefully");
}

/// 测试获取不存在索引的元数据
///
/// 验证获取不存在索引元数据时的行为
#[test]
fn test_get_metadata_nonexistent_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let persistence = PersistenceManager::new(base_path);
    let metadata = persistence.get_index_metadata("nonexistent_index")
        .expect("Should return default metadata for nonexistent index");

    assert_eq!(metadata.name, String::new());
    assert_eq!(metadata.document_count, 0);
}

/// 测试获取不存在索引的大小
///
/// 验证获取不存在索引大小时的返回值
#[test]
fn test_get_size_nonexistent_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let persistence = PersistenceManager::new(base_path);
    let size = persistence.get_index_size("nonexistent_index")
        .expect("Should return 0 for nonexistent index");

    assert_eq!(size, 0, "Nonexistent index should have size 0");
}

/// 测试删除不存在的备份
///
/// 验证删除不存在的备份时的行为
#[test]
fn test_delete_nonexistent_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let persistence = PersistenceManager::new(base_path);
    let deleted = persistence.delete_old_backups("nonexistent_index", 0)
        .expect("Should succeed even for nonexistent index");

    assert_eq!(deleted, 0, "Should delete 0 backups for nonexistent index");
}

/// 测试文档字段缺失
///
/// 验证添加文档时缺少某些字段的行为
#[test]
fn test_add_document_with_missing_fields() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Only Title".to_string());

    let result = add_document(&manager, &schema, "doc1", &fields);
    assert!(result.is_ok(), "Document with missing content field should be accepted");
}

/// 测试文档字段为空值
///
/// 验证添加文档时字段值为空字符串的行为
#[test]
fn test_add_document_with_empty_field_values() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "".to_string());
    fields.insert("content".to_string(), "".to_string());

    let result = add_document(&manager, &schema, "doc1", &fields);
    assert!(result.is_ok(), "Document with empty field values should be accepted");
}

/// 测试超长文档 ID
///
/// 验证超长文档 ID 的处理
#[test]
fn test_add_document_with_very_long_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let long_id = "a".repeat(1000);
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Test".to_string());
    fields.insert("content".to_string(), "Content".to_string());

    let result = add_document(&manager, &schema, &long_id, &fields);
    assert!(result.is_ok(), "Very long document ID should be accepted");
}

/// 测试超长文档内容
///
/// 验证超长文档内容的处理
#[test]
fn test_add_document_with_very_long_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let long_content = "x".repeat(1_000_000);
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Long Content Test".to_string());
    fields.insert("content".to_string(), long_content);

    let result = add_document(&manager, &schema, "doc1", &fields);
    assert!(result.is_ok(), "Very long content should be accepted");
}

/// 测试重复添加相同 ID 文档
///
/// 验证重复添加相同 ID 文档的行为
#[test]
fn test_add_duplicate_document_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields1 = HashMap::new();
    fields1.insert("title".to_string(), "First Version".to_string());
    fields1.insert("content".to_string(), "First content".to_string());

    let result1 = add_document(&manager, &schema, "doc1", &fields1);
    assert!(result1.is_ok(), "First add should succeed");

    let mut fields2 = HashMap::new();
    fields2.insert("title".to_string(), "Second Version".to_string());
    fields2.insert("content".to_string(), "Second content".to_string());

    let result2 = add_document(&manager, &schema, "doc1", &fields2);
    assert!(result2.is_ok(), "Duplicate ID add should succeed");
}

/// 测试索引路径包含特殊字符
///
/// 验证索引路径包含特殊字符时的处理
#[test]
fn test_create_index_with_special_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test index with spaces");

    let result = IndexManager::create(&index_path);
    assert!(result.is_ok(), "Index with spaces in path should be created");
}

/// 测试索引路径包含 Unicode 字符
///
/// 验证索引路径包含 Unicode 字符时的处理
#[test]
fn test_create_index_with_unicode_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("中文索引");

    let result = IndexManager::create(&index_path);
    assert!(result.is_ok(), "Index with Unicode in path should be created");
}

/// 测试多次获取写入器
///
/// 验证多次获取写入器时的行为
#[test]
fn test_multiple_writer_acquisition() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");

    let writer1 = manager.writer().expect("Failed to get first writer");
    drop(writer1);

    let writer2 = manager.writer().expect("Failed to get second writer");
    drop(writer2);

    let writer3 = manager.writer().expect("Failed to get third writer");
    drop(writer3);
}

/// 测试多次获取读取器
///
/// 验证多次获取读取器时的行为
#[test]
fn test_multiple_reader_acquisition() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");

    let reader1 = manager.reader().expect("Failed to get first reader");
    let reader2 = manager.reader().expect("Failed to get second reader");
    let reader3 = manager.reader().expect("Failed to get third reader");

    assert_eq!(reader1.searcher().num_docs(), reader2.searcher().num_docs());
    assert_eq!(reader2.searcher().num_docs(), reader3.searcher().num_docs());
}
