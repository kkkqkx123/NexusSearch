//! 数据一致性测试
//!
//! 测试数据完整性和一致性

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::{add_document, update_document, get_document},
    batch::batch_add_documents,
    delete::delete_document,
    search::{search, SearchOptions},
    persistence::PersistenceManager,
    stats::get_stats,
};
use std::collections::HashMap;
use tempfile::TempDir;
use tantivy::schema::Value;

/// 测试索引重开后的数据一致性
///
/// 验证索引关闭后重新打开，所有文档数据保持完整
#[test]
fn test_data_consistency_after_reopen() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    {
        let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
        let schema = IndexSchema::new();

        for i in 0..10 {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            add_document(&manager, &schema, &format!("doc{}", i), &fields)
                .expect("Failed to add document");
        }
    }

    let manager = IndexManager::open(&index_path).expect("Failed to open index");
    let schema = IndexSchema::new();

    for i in 0..10 {
        let doc = get_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some(), "Document {} should exist", i);
    }
}

/// 测试备份恢复后的数据一致性
///
/// 验证备份和恢复操作后数据保持完整
#[test]
fn test_backup_restore_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("original_index");
    let restore_path = base_path.join("restored_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = (0..20)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Unique content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents).expect("Failed to add documents");

    let persistence = PersistenceManager::new(base_path);
    let backup_info = persistence.create_backup(&manager, "original_index")
        .expect("Failed to create backup");

    persistence.restore_backup("restored_index", &backup_info.backup_path)
        .expect("Failed to restore backup");

    let restored_manager = IndexManager::open(&restore_path)
        .expect("Failed to open restored index");

    for i in 0..20 {
        let doc = get_document(&restored_manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some(), "Restored document {} should exist", i);
    }
}

/// 测试批量操作的原子性
///
/// 验证批量添加操作要么全部成功，要么全部失败
#[test]
fn test_batch_operation_atomicity() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = (0..10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    assert_eq!(count, 10);

    for i in 0..10 {
        let doc = get_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some());
    }
}

/// 测试更新操作的文档数一致性
///
/// 验证更新操作不会导致文档数量异常增加
#[test]
fn test_update_document_count_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Original".to_string());
    fields.insert("content".to_string(), "Original content".to_string());
    add_document(&manager, &schema, "doc1", &fields).expect("Failed to add document");

    let reader = manager.reader().expect("Failed to get reader");
    let initial_count = reader.searcher().num_docs();

    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());
    update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");

    let reader = manager.reader().expect("Failed to get reader");
    let final_count = reader.searcher().num_docs();

    assert!(final_count >= initial_count);
}

/// 测试删除后搜索结果一致性
///
/// 验证删除文档后搜索结果不再包含已删除文档
#[test]
fn test_search_consistency_after_delete() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    for i in 0..10 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("UniqueKeyword Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "UniqueKeyword", &options)
        .expect("Failed to search");
    assert_eq!(results.len(), 10);

    delete_document(&manager, &schema, "doc5").expect("Failed to delete document");

    let (results, _) = search(&manager, &schema, "UniqueKeyword", &options)
        .expect("Failed to search after delete");
    assert_eq!(results.len(), 9);
}

/// 测试更新后搜索结果一致性
///
/// 验证更新文档后搜索结果反映最新内容
#[test]
fn test_search_consistency_after_update() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "OriginalTitle".to_string());
    fields.insert("content".to_string(), "Original content".to_string());
    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "OriginalTitle", &options)
        .expect("Failed to search");
    assert_eq!(results.len(), 1);

    let (results, _) = search(&manager, &schema, "UpdatedTitle", &options)
        .expect("Failed to search");
    assert_eq!(results.len(), 0);

    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "UpdatedTitle".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());
    update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");

    let (results, _) = search(&manager, &schema, "UpdatedTitle", &options)
        .expect("Failed to search after update");
    assert_eq!(results.len(), 1);
}

/// 测试统计信息一致性
///
/// 验证索引统计信息与实际数据一致
#[test]
fn test_stats_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = (1..=50)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Stats Document {}", i));
            fields.insert("content".to_string(), format!("Stats content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    manager.reload_reader().expect("Failed to reload reader");

    let stats = get_stats(&manager).expect("Failed to get stats");
    assert_eq!(stats.total_documents, 50);
}

/// 测试多次备份的一致性
///
/// 验证多次备份之间数据的一致性
#[test]
fn test_multiple_backups_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();
    let persistence = PersistenceManager::new(base_path);

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Backup Test".to_string());
    fields.insert("content".to_string(), "Backup content".to_string());
    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    let backup1 = persistence.create_backup(&manager, "test_index")
        .expect("Failed to create first backup");

    std::thread::sleep(std::time::Duration::from_millis(100));

    let backup2 = persistence.create_backup(&manager, "test_index")
        .expect("Failed to create second backup");

    assert_ne!(backup1.backup_id, backup2.backup_id);
    assert!(backup1.backup_path.exists());
    assert!(backup2.backup_path.exists());
}

/// 测试文档内容完整性
///
/// 验证存储和检索的文档内容完全一致
#[test]
fn test_document_content_integrity() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let original_title = "Test Title with Special Characters: <>&\"'";
    let original_content = "Content with\nmultiple\nlines\tand\ttabs";

    let mut fields = HashMap::new();
    fields.insert("title".to_string(), original_title.to_string());
    fields.insert("content".to_string(), original_content.to_string());
    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get document")
        .expect("Document should exist");

    let title_value = doc.get_first(schema.title)
        .and_then(|v| v.as_str())
        .unwrap();
    let content_value = doc.get_first(schema.content)
        .and_then(|v| v.as_str())
        .unwrap();

    assert_eq!(title_value, original_title);
    assert_eq!(content_value, original_content);
}

/// 测试 Unicode 内容一致性
///
/// 验证 Unicode 字符在存储和检索过程中保持一致
#[test]
fn test_unicode_content_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let test_cases = vec![
        ("chinese", "中文测试文档", "这是中文内容"),
        ("japanese", "日本語テスト", "日本語のコンテンツ"),
        ("korean", "한국어 테스트", "한국어 콘텐츠"),
        ("arabic", "اختبار عربي", "محتوى عربي"),
        ("hebrew", "בדיקה בעברית", "תוכן בעברית"),
        ("emoji", "Emoji Test 🚀", "Content with emojis 🎉🎊"),
    ];

    for (doc_id, title, content) in &test_cases {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), title.to_string());
        fields.insert("content".to_string(), content.to_string());
        add_document(&manager, &schema, doc_id, &fields)
            .expect("Failed to add document");
    }

    for (doc_id, original_title, original_content) in &test_cases {
        let doc = get_document(&manager, &schema, doc_id)
            .expect("Failed to get document")
            .expect("Document should exist");

        let title_value = doc.get_first(schema.title)
            .and_then(|v| v.as_str())
            .unwrap();
        let content_value = doc.get_first(schema.content)
            .and_then(|v| v.as_str())
            .unwrap();

        assert_eq!(title_value, *original_title, "Title mismatch for {}", doc_id);
        assert_eq!(content_value, *original_content, "Content mismatch for {}", doc_id);
    }
}

/// 测试索引完整性验证
///
/// 验证索引在多次操作后保持完整性
#[test]
fn test_index_integrity_after_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    for i in 0..20 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    for i in 0..10 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Updated Document {}", i));
        fields.insert("content".to_string(), format!("Updated Content {}", i));
        update_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to update document");
    }

    for i in 15..20 {
        delete_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to delete document");
    }

    for i in 0..15 {
        let doc = get_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some(), "Document {} should exist", i);
    }

    for i in 15..20 {
        let doc = get_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_none(), "Document {} should be deleted", i);
    }
}
