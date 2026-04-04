//! 文档操作集成测试
//!
//! 测试文档的添加、更新、删除和获取功能

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::{add_document, update_document, get_document},
    delete::delete_document,
};
use std::collections::HashMap;
use tempfile::TempDir;
use tantivy::schema::Value;

#[test]
fn test_add_single_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "First Document".to_string());
    fields.insert("content".to_string(), "This is the content of first document".to_string());

    let result = add_document(&manager, &schema, "doc1", &fields);
    assert!(result.is_ok(), "Should successfully add document");
}

#[test]
fn test_add_multiple_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加多个文档
    for i in 1..=5 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content for document {}", i));

        let result = add_document(&manager, &schema, &format!("doc{}", i), &fields);
        assert!(result.is_ok(), "Should successfully add document {}", i);
    }

    // 验证文档数量
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 5, "Should have 5 documents");
}

#[test]
fn test_get_existing_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Test Document".to_string());
    fields.insert("content".to_string(), "Test content for retrieval".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 获取文档
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get document");

    assert!(doc.is_some(), "Document should exist");
    let doc = doc.unwrap();

    // 验证文档内容
    let doc_id_value = doc.get_first(schema.document_id)
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(doc_id_value, "doc1", "Document ID should match");
}

#[test]
fn test_get_nonexistent_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 尝试获取不存在的文档
    let doc = get_document(&manager, &schema, "nonexistent")
        .expect("Failed to query document");

    assert!(doc.is_none(), "Nonexistent document should return None");
}

#[test]
fn test_update_existing_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Original Title".to_string());
    fields.insert("content".to_string(), "Original content".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 更新文档
    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated Title".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());

    update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");

    // 验证文档已更新
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get updated document");

    assert!(doc.is_some());
    let doc = doc.unwrap();

    let title_value = doc.get_first(schema.title)
        .and_then(|v| v.as_str())
        .unwrap();
    assert_eq!(title_value, "Updated Title", "Title should be updated");
}

#[test]
fn test_update_nonexistent_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 尝试更新不存在的文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "New Document".to_string());
    fields.insert("content".to_string(), "New content".to_string());

    let result = update_document(&manager, &schema, "nonexistent", &fields);
    assert!(result.is_ok(), "Update should succeed even for nonexistent document");

    // 验证文档已被创建
    let doc = get_document(&manager, &schema, "nonexistent")
        .expect("Failed to get document");

    assert!(doc.is_some(), "Document should be created by update operation");
}

#[test]
fn test_delete_existing_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "To Delete".to_string());
    fields.insert("content".to_string(), "Content to delete".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 删除文档
    delete_document(&manager, &schema, "doc1")
        .expect("Failed to delete document");

    // 验证文档已删除
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to query document");

    assert!(doc.is_none(), "Document should be deleted");
}

#[test]
fn test_delete_nonexistent_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 删除不存在的文档
    let result = delete_document(&manager, &schema, "nonexistent");
    assert!(result.is_ok(), "Delete should succeed even for nonexistent document");
}

#[test]
fn test_document_with_unicode_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加包含 Unicode 内容的文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Unicode 测试 🚀".to_string());
    fields.insert("content".to_string(), "Hello 世界 Привет مرحبا".to_string());

    let result = add_document(&manager, &schema, "unicode_doc", &fields);
    assert!(result.is_ok(), "Should handle Unicode content");

    // 验证可以检索
    let doc = get_document(&manager, &schema, "unicode_doc")
        .expect("Failed to get Unicode document");

    assert!(doc.is_some());
    let doc = doc.unwrap();

    let title_value = doc.get_first(schema.title)
        .and_then(|v| v.as_str())
        .unwrap();
    assert!(title_value.contains("测试"), "Should preserve Unicode characters");
}

#[test]
fn test_document_with_empty_fields() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加包含空字段的文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "".to_string());
    fields.insert("content".to_string(), "".to_string());

    let result = add_document(&manager, &schema, "empty_doc", &fields);
    assert!(result.is_ok(), "Should handle empty fields");

    // 验证文档存在
    let doc = get_document(&manager, &schema, "empty_doc")
        .expect("Failed to get document");

    assert!(doc.is_some());
}

#[test]
fn test_document_with_long_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加包含长内容的文档
    let long_content = "A".repeat(10000);
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Long Content Document".to_string());
    fields.insert("content".to_string(), long_content);

    let result = add_document(&manager, &schema, "long_doc", &fields);
    assert!(result.is_ok(), "Should handle long content");

    // 验证文档存在
    let doc = get_document(&manager, &schema, "long_doc")
        .expect("Failed to get document");

    assert!(doc.is_some());
}

#[test]
fn test_document_with_special_characters() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加包含特殊字符的文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Special <>&\"' Chars".to_string());
    fields.insert("content".to_string(), "Content with \n\t\r special chars".to_string());

    let result = add_document(&manager, &schema, "special_doc", &fields);
    assert!(result.is_ok(), "Should handle special characters");

    // 验证文档存在
    let doc = get_document(&manager, &schema, "special_doc")
        .expect("Failed to get document");

    assert!(doc.is_some());
}

#[test]
fn test_document_with_multilingual_content() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加多语言文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Multilingual Document".to_string());
    fields.insert("content".to_string(), "English 中文 日本語 한국어 العربية עברית".to_string());

    let result = add_document(&manager, &schema, "multilingual_doc", &fields);
    assert!(result.is_ok(), "Should handle multilingual content");

    // 验证文档存在
    let doc = get_document(&manager, &schema, "multilingual_doc")
        .expect("Failed to get document");

    assert!(doc.is_some());
}

#[test]
fn test_document_update_preserves_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Original".to_string());
    fields.insert("content".to_string(), "Original".to_string());

    add_document(&manager, &schema, "preserve_id", &fields)
        .expect("Failed to add document");

    // 更新文档
    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated".to_string());
    updated_fields.insert("content".to_string(), "Updated".to_string());

    update_document(&manager, &schema, "preserve_id", &updated_fields)
        .expect("Failed to update document");

    // 验证文档数量没有增加
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 1, "Should still have only 1 document after update");
}
