//! 批量操作集成测试
//!
//! 测试批量添加、更新和删除文档的功能

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    batch::{batch_add_documents, batch_update_documents, batch_add_documents_optimized},
    delete::batch_delete_documents,
};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_batch_add_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 准备批量文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Batch Document {}", i));
            fields.insert("content".to_string(), format!("Batch content for document {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    // 批量添加
    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to batch add documents");

    assert_eq!(count, 10, "Should add 10 documents");

    // 验证文档数量
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 10, "Should have 10 documents in index");
}

#[test]
fn test_batch_add_empty_list() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = Vec::new();

    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to batch add empty list");

    assert_eq!(count, 0, "Should add 0 documents");
}

#[test]
fn test_batch_add_single_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Single Document".to_string());
    fields.insert("content".to_string(), "Single document content".to_string());

    let documents = vec![("doc1".to_string(), fields)];

    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to batch add single document");

    assert_eq!(count, 1, "Should add 1 document");
}

#[test]
fn test_batch_update_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 先添加文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=5)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Original content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add initial documents");

    // 批量更新文档
    let updated_documents: Vec<(String, HashMap<String, String>)> = (1..=5)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Updated Document {}", i));
            fields.insert("content".to_string(), format!("Updated content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    let count = batch_update_documents(&manager, &schema, updated_documents)
        .expect("Failed to batch update documents");

    assert_eq!(count, 5, "Should update 5 documents");

    // 验证文档数量保持不变
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 5, "Should still have 5 documents after update");
}

#[test]
fn test_batch_add_optimized_with_batch_size() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 准备 20 个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=20)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Optimized Document {}", i));
            fields.insert("content".to_string(), format!("Optimized content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    // 使用批量大小为 5 进行优化添加
    let count = batch_add_documents_optimized(&manager, &schema, documents, 5)
        .expect("Failed to batch add optimized");

    assert_eq!(count, 20, "Should add 20 documents");

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 20, "Should have 20 documents in index");
}

#[test]
fn test_batch_add_optimized_with_large_batch_size() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 准备 10 个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Large Batch Document {}", i));
            fields.insert("content".to_string(), format!("Large batch content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    // 使用批量大小为 100（大于文档数量）
    let count = batch_add_documents_optimized(&manager, &schema, documents, 100)
        .expect("Failed to batch add optimized");

    assert_eq!(count, 10, "Should add 10 documents");
}

#[test]
fn test_batch_delete_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 先添加文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add initial documents");

    // 批量删除文档
    let doc_ids: Vec<String> = vec!["doc1".to_string(), "doc3".to_string(), "doc5".to_string()];

    let count = batch_delete_documents(&manager, &schema, &doc_ids)
        .expect("Failed to batch delete documents");

    assert_eq!(count, 3, "Should delete 3 documents");

    // 验证剩余文档数量
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 7, "Should have 7 documents after deletion");
}

#[test]
fn test_batch_delete_empty_list() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let doc_ids: Vec<String> = Vec::new();

    let count = batch_delete_documents(&manager, &schema, &doc_ids)
        .expect("Failed to batch delete empty list");

    assert_eq!(count, 0, "Should delete 0 documents");
}

#[test]
fn test_batch_delete_nonexistent_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let doc_ids: Vec<String> = vec!["nonexistent1".to_string(), "nonexistent2".to_string()];

    let count = batch_delete_documents(&manager, &schema, &doc_ids)
        .expect("Failed to batch delete nonexistent documents");

    assert_eq!(count, 2, "Should report 2 deletions even if documents don't exist");
}

#[test]
fn test_batch_add_large_number_of_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 准备 100 个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=100)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content for document {}", i));
            (format!("doc{:04}", i), fields)
        })
        .collect();

    let count = batch_add_documents_optimized(&manager, &schema, documents, 20)
        .expect("Failed to batch add large number of documents");

    assert_eq!(count, 100, "Should add 100 documents");

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 100, "Should have 100 documents in index");
}

#[test]
fn test_batch_add_with_duplicate_ids() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加包含重复 ID 的文档
    let documents: Vec<(String, HashMap<String, String>)> = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "First Document".to_string());
            fields.insert("content".to_string(), "First content".to_string());
            fields
        }),
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Duplicate Document".to_string());
            fields.insert("content".to_string(), "Duplicate content".to_string());
            fields
        }),
    ];

    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to batch add documents with duplicate IDs");

    assert_eq!(count, 2, "Should report 2 additions");

    // 验证文档数量
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    // 注意：Tantivy 可能会为相同 ID 的文档创建多个文档
    assert!(searcher.num_docs() >= 1, "Should have at least 1 document");
}

#[test]
fn test_batch_operations_with_special_characters() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加包含特殊字符的文档
    let documents: Vec<(String, HashMap<String, String>)> = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Special & <Characters>".to_string());
            fields.insert("content".to_string(), "Content with \"quotes\" and 'apostrophes'".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Unicode 🚀 Test".to_string());
            fields.insert("content".to_string(), "中文 日本語 한국어".to_string());
            fields
        }),
    ];

    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to batch add documents with special characters");

    assert_eq!(count, 2, "Should add 2 documents with special characters");

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 2, "Should have 2 documents");
}

#[test]
fn test_batch_update_with_new_ids() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 批量更新不存在的文档 ID（应该创建新文档）
    let documents: Vec<(String, HashMap<String, String>)> = (1..=3)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("New Document {}", i));
            fields.insert("content".to_string(), format!("New content {}", i));
            (format!("new_doc{}", i), fields)
        })
        .collect();

    let count = batch_update_documents(&manager, &schema, documents)
        .expect("Failed to batch update with new IDs");

    assert_eq!(count, 3, "Should create 3 new documents");

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 3, "Should have 3 new documents");
}

#[test]
fn test_batch_mixed_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 第一步：批量添加
    let documents1: Vec<(String, HashMap<String, String>)> = (1..=5)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents1)
        .expect("Failed to add documents");

    // 第二步：批量更新
    let updated_docs: Vec<(String, HashMap<String, String>)> = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Updated Doc 1".to_string());
            fields.insert("content".to_string(), "Updated content 1".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Updated Doc 2".to_string());
            fields.insert("content".to_string(), "Updated content 2".to_string());
            fields
        }),
    ];

    batch_update_documents(&manager, &schema, updated_docs)
        .expect("Failed to update documents");

    // 第三步：批量删除
    let doc_ids = vec!["doc3".to_string(), "doc4".to_string()];
    batch_delete_documents(&manager, &schema, &doc_ids)
        .expect("Failed to delete documents");

    // 第四步：批量添加更多文档
    let documents2: Vec<(String, HashMap<String, String>)> = (6..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("New Document {}", i));
            fields.insert("content".to_string(), format!("New content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents2)
        .expect("Failed to add more documents");

    // 验证最终文档数量
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 8, "Should have 8 documents after mixed operations");
}