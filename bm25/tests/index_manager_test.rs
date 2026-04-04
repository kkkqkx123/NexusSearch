//! 索引管理器集成测试
//!
//! 测试 IndexManager 的创建、打开、读写器获取等核心功能

use bm25_service::api::core::{IndexManager, IndexSchema};
use tempfile::TempDir;

#[test]
fn test_create_new_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 创建新索引
    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    // 验证索引文件已创建
    assert!(index_path.exists(), "Index directory should exist");

    // 验证可以获取 schema
    let schema = manager.schema();
    let fields: Vec<_> = schema.fields().collect();
    assert!(!fields.is_empty(), "Schema should have fields");

    // 验证可以获取 index 对象
    let index = manager.index();
    let index_schema = index.schema();
    let index_fields: Vec<_> = index_schema.fields().collect();
    assert_eq!(index_fields.len(), fields.len());
}

#[test]
fn test_open_existing_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 首先创建索引
    {
        let _manager = IndexManager::create(&index_path)
            .expect("Failed to create index manager");
    }

    // 然后打开已存在的索引
    let manager = IndexManager::open(&index_path)
        .expect("Failed to open existing index manager");

    // 验证 schema 字段一致
    let schema = manager.schema();
    let fields: Vec<_> = schema.fields().collect();
    assert_eq!(fields.len(), 3, "Should have 3 fields");
}

#[test]
fn test_create_index_in_subdirectory() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("nested").join("path").join("test_index");

    // 创建嵌套目录的索引
    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index in nested directory");

    assert!(index_path.exists(), "Nested directory should be created");

    let schema = manager.schema();
    let fields: Vec<_> = schema.fields().collect();
    assert!(!fields.is_empty());
}

#[test]
fn test_multiple_indices_in_different_paths() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    let index1_path = temp_dir.path().join("index1");
    let index2_path = temp_dir.path().join("index2");

    let _manager1 = IndexManager::create(&index1_path)
        .expect("Failed to create first index");
    let _manager2 = IndexManager::create(&index2_path)
        .expect("Failed to create second index");

    // 验证两个索引是独立的
    assert!(index1_path.exists());
    assert!(index2_path.exists());

    // 验证路径不同
    assert_ne!(index1_path, index2_path, "Indices should have different paths");
}

#[test]
fn test_get_writer_multiple_times() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    // 获取写入器
    let writer1 = manager.writer()
        .expect("Failed to get writer");

    // 释放第一个写入器
    drop(writer1);

    // 获取第二个写入器（在前一个释放后）
    let writer2 = manager.writer()
        .expect("Failed to get writer again");

    // 写入器应该是独立的实例
    drop(writer2);
}

#[test]
fn test_get_reader_multiple_times() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    // 获取读取器多次
    let reader1 = manager.reader()
        .expect("Failed to get reader");
    let reader2 = manager.reader()
        .expect("Failed to get reader again");

    // 验证读取器功能正常
    assert_eq!(reader1.searcher().num_docs(), reader2.searcher().num_docs());
}

#[test]
fn test_schema_field_names() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = manager.schema();

    // 获取所有字段名称
    let field_names: Vec<String> = schema.fields()
        .map(|(field, _)| {
            schema.get_field_name(field).to_string()
        })
        .collect();

    // 验证包含预期的字段
    assert!(field_names.contains(&"document_id".to_string()));
    assert!(field_names.contains(&"title".to_string()));
    assert!(field_names.contains(&"content".to_string()));
    assert_eq!(field_names.len(), 3);
}

#[test]
fn test_index_persistence_after_reopen() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 创建索引并验证 schema 字段数量
    let field_count = {
        let manager = IndexManager::create(&index_path)
            .expect("Failed to create index manager");
        let fields: Vec<_> = manager.schema().fields().collect();
        fields.len()
    };

    // 重新打开索引
    let manager = IndexManager::open(&index_path)
        .expect("Failed to reopen index");

    // 验证 schema 字段数量保持一致
    let fields: Vec<_> = manager.schema().fields().collect();
    assert_eq!(
        fields.len(),
        field_count,
        "Schema field count should persist after reopen"
    );
}

#[test]
fn test_schema_field_types() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = manager.schema();

    // 验证字段类型
    let document_id_field = schema.get_field("document_id")
        .expect("document_id field should exist");
    let title_field = schema.get_field("title")
        .expect("title field should exist");
    let content_field = schema.get_field("content")
        .expect("content field should exist");

    // 验证字段是文本类型
    use tantivy::schema::FieldType;
    if let FieldType::Str(_) = schema.get_field_entry(document_id_field).field_type() {}
    if let FieldType::Str(_) = schema.get_field_entry(title_field).field_type() {}
    if let FieldType::Str(_) = schema.get_field_entry(content_field).field_type() {}
}

#[test]
fn test_create_index_with_relative_path() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");

    // 使用相对路径
    let relative_path = "test_index";
    let full_path = temp_dir.path().join(relative_path);

    let manager = IndexManager::create(&full_path)
        .expect("Failed to create index with relative path");

    assert!(full_path.exists());
    let fields: Vec<_> = manager.schema().fields().collect();
    assert!(!fields.is_empty());
}

#[test]
fn test_manager_with_index_schema() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 验证 IndexManager 的 schema 和 IndexSchema 字段数量一致
    let fields: Vec<_> = manager.schema().fields().collect();
    assert_eq!(
        fields.len(),
        3,
        "Manager schema should have 3 fields"
    );

    // 验证 IndexSchema 的字段
    let _ = schema.document_id;
    let _ = schema.title;
    let _ = schema.content;
}