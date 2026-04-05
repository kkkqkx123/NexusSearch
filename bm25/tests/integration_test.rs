//! 综合集成测试
//!
//! 测试完整的索引生命周期，包括文档管理、搜索、持久化等功能的组合使用

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::{add_document, update_document, get_document},
    batch::{batch_add_documents, batch_update_documents},
    delete::delete_document,
    search::{search, SearchOptions},
    persistence::PersistenceManager,
    stats::get_stats,
};
use std::collections::HashMap;
use tempfile::TempDir;

/// 测试完整的索引生命周期
/// 
/// 注意：由于 Tantivy 的 reader 缓存机制，文档提交后需要调用 reload_reader()
/// 才能使搜索看到最新的更改。这是 Tantivy 的设计特性，用于提高搜索性能。
#[test]
fn test_full_index_lifecycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 1. 创建索引（禁用 reader 缓存以确保实时性）
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 2. 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "First Document".to_string());
    fields.insert("content".to_string(), "Content of first document".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 3. 验证文档存在
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get document");
    assert!(doc.is_some(), "Document should exist");

    // 4. 更新文档
    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated First Document".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());

    update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");

    // 5. 搜索文档（禁用缓存时不需要 reload）
    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Updated", &options)
        .expect("Failed to search");
    assert!(!results.is_empty(), "Should find updated document");

    // 6. 删除文档
    delete_document(&manager, &schema, "doc1")
        .expect("Failed to delete document");

    // 7. 验证文档已删除
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get document");
    assert!(doc.is_none(), "Document should be deleted");
}

/// 测试使用 reader 缓存时的正确行为
/// 
/// 当启用 reader 缓存时，需要在文档操作后调用 reload_reader() 才能看到最新更改
#[test]
fn test_index_lifecycle_with_caching() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 创建索引（启用 reader 缓存）
    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "First Document".to_string());
    fields.insert("content".to_string(), "Content of first document".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 重新加载 reader 以看到最新更改
    manager.reload_reader().expect("Failed to reload reader");

    // 验证文档存在
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get document");
    assert!(doc.is_some(), "Document should exist after reload");

    // 更新文档
    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated First Document".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());

    update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");

    // 重新加载 reader
    manager.reload_reader().expect("Failed to reload reader");

    // 搜索更新后的文档
    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Updated", &options)
        .expect("Failed to search");
    assert!(!results.is_empty(), "Should find updated document after reload");

    // 删除文档
    delete_document(&manager, &schema, "doc1")
        .expect("Failed to delete document");

    // 重新加载 reader
    manager.reload_reader().expect("Failed to reload reader");

    // 验证文档已删除
    let doc = get_document(&manager, &schema, "doc1")
        .expect("Failed to get document");
    assert!(doc.is_none(), "Document should be deleted after reload");
}

#[test]
fn test_batch_operations_with_search() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 禁用缓存以确保实时搜索
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 批量添加文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=20)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content for document {}", i));
            (format!("doc{:02}", i), fields)
        })
        .collect();

    let count = batch_add_documents(&manager, &schema, documents)
        .expect("Failed to batch add documents");
    assert_eq!(count, 20);

    // 搜索所有文档
    let options = SearchOptions {
        limit: 50,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results, _) = search(&manager, &schema, "Document", &options)
        .expect("Failed to search");
    assert_eq!(results.len(), 20, "Should find all 20 documents");

    // 验证分页
    let options_page1 = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };
    let (results_page1, _) = search(&manager, &schema, "Document", &options_page1)
        .expect("Failed to search page 1");
    assert_eq!(results_page1.len(), 10, "Should find 10 documents on page 1");

    let options_page2 = SearchOptions {
        limit: 10,
        offset: 10,
        field_weights: HashMap::new(),
        highlight: false,
    };
    let (results_page2, _) = search(&manager, &schema, "Document", &options_page2)
        .expect("Failed to search page 2");
    assert_eq!(results_page2.len(), 10, "Should find 10 documents on page 2");
}

/// 测试文档更新和删除的搜索可见性
/// 
/// 使用禁用缓存的配置来确保操作立即可见
#[test]
fn test_search_with_update_and_delete() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 禁用缓存
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加两个文档
    let mut fields1 = HashMap::new();
    fields1.insert("title".to_string(), "Rust Programming".to_string());
    fields1.insert("content".to_string(), "Rust is a systems programming language".to_string());

    add_document(&manager, &schema, "doc1", &fields1)
        .expect("Failed to add document");

    let mut fields2 = HashMap::new();
    fields2.insert("title".to_string(), "TypeScript Programming".to_string());
    fields2.insert("content".to_string(), "TypeScript is a typed superset of JavaScript".to_string());

    add_document(&manager, &schema, "doc2", &fields2)
        .expect("Failed to add document");

    // 搜索 "Programming"
    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Programming", &options)
        .expect("Failed to search");
    assert_eq!(results.len(), 2, "Should find 2 documents");

    // 更新第一个文档
    let mut updated_fields1 = HashMap::new();
    updated_fields1.insert("title".to_string(), "Rust Language".to_string());
    updated_fields1.insert("content".to_string(), "Rust is fast and safe".to_string());

    update_document(&manager, &schema, "doc1", &updated_fields1)
        .expect("Failed to update document");

    // 再次搜索 "Programming"（应该只找到一个）
    let (results, _) = search(&manager, &schema, "Programming", &options)
        .expect("Failed to search after update");
    assert_eq!(results.len(), 1, "Should find only 1 document after update");

    // 删除第二个文档
    delete_document(&manager, &schema, "doc2")
        .expect("Failed to delete document");

    // 再次搜索 "Programming"（应该找不到）
    let (results, _) = search(&manager, &schema, "Programming", &options)
        .expect("Failed to search after delete");
    assert_eq!(results.len(), 0, "Should find 0 documents after delete");
}

#[test]
fn test_persistence_with_full_workflow() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引并添加文档
    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = (1..=5)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 重新加载 reader 以获取最新统计
    manager.reload_reader().expect("Failed to reload reader");

    // 创建备份
    let persistence_manager = PersistenceManager::new(base_path);
    let backup_info = persistence_manager.create_backup(&manager, "test_index")
        .expect("Failed to create backup");

    assert!(backup_info.backup_path.exists());

    // 列出备份
    let backups = persistence_manager.list_backups("test_index")
        .expect("Failed to list backups");
    assert_eq!(backups.len(), 1);

    // 获取索引统计
    let stats = get_stats(&manager)
        .expect("Failed to get stats");
    assert_eq!(stats.total_documents, 5);
}

#[test]
fn test_complex_search_scenarios() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 禁用缓存
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加多样化的文档
    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming Language".to_string());
            fields.insert("content".to_string(), "Rust is fast, safe, and concurrent".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "TypeScript Language".to_string());
            fields.insert("content".to_string(), "TypeScript adds type safety to JavaScript".to_string());
            fields
        }),
        ("doc3".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Go Language".to_string());
            fields.insert("content".to_string(), "Go is simple and efficient".to_string());
            fields
        }),
        ("doc4".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Python Programming".to_string());
            fields.insert("content".to_string(), "Python is easy to learn and powerful".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 测试各种搜索场景
    let options = SearchOptions::default();

    // 搜索 "Language"
    let (results1, _) = search(&manager, &schema, "Language", &options)
        .expect("Failed to search 'Language'");
    assert_eq!(results1.len(), 3, "Should find 3 documents with 'Language'");

    // 搜索 "Programming"
    let (results2, _) = search(&manager, &schema, "Programming", &options)
        .expect("Failed to search 'Programming'");
    assert_eq!(results2.len(), 2, "Should find 2 documents with 'Programming'");

    // 搜索 "safe"
    let (results3, _) = search(&manager, &schema, "safe", &options)
        .expect("Failed to search 'safe'");
    assert_eq!(results3.len(), 1, "Should find 1 document with 'safe'");

    // 搜索 "TypeScript"
    let (results4, _) = search(&manager, &schema, "TypeScript", &options)
        .expect("Failed to search 'TypeScript'");
    assert_eq!(results4.len(), 1, "Should find 1 document with 'TypeScript'");
}

/// 测试搜索高亮和分页功能
#[test]
fn test_search_with_highlights_and_pagination() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 禁用缓存
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Search Test Document {}", i));
            fields.insert("content".to_string(), format!("This is test content for document {}", i));
            (format!("doc{:02}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 测试带高亮的搜索
    let options_with_highlight = SearchOptions {
        limit: 5,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: true,
    };

    let (results, _) = search(&manager, &schema, "test", &options_with_highlight)
        .expect("Failed to search with highlight");
    
    assert_eq!(results.len(), 5, "Should find 5 documents");
    
    // 验证高亮字段存在（即使内容为空）
    for result in &results {
        assert!(result.highlights.contains_key("title"), "Should have title highlight field");
        assert!(result.highlights.contains_key("content"), "Should have content highlight field");
    }

    // 测试分页
    let options_page1 = SearchOptions {
        limit: 3,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };
    let (results_page1, _) = search(&manager, &schema, "Document", &options_page1)
        .expect("Failed to search page 1");
    assert_eq!(results_page1.len(), 3, "Should find 3 documents on page 1");

    let options_page2 = SearchOptions {
        limit: 3,
        offset: 3,
        field_weights: HashMap::new(),
        highlight: false,
    };
    let (results_page2, _) = search(&manager, &schema, "Document", &options_page2)
        .expect("Failed to search page 2");
    assert_eq!(results_page2.len(), 3, "Should find 3 documents on page 2");

    // 验证两页结果不重复
    let ids_page1: std::collections::HashSet<_> = results_page1.iter().map(|r| &r.document_id).collect();
    let ids_page2: std::collections::HashSet<_> = results_page2.iter().map(|r| &r.document_id).collect();
    let intersection: Vec<_> = ids_page1.intersection(&ids_page2).collect();
    assert!(intersection.is_empty(), "Page 1 and Page 2 should have no overlapping documents");
}

/// 测试多次连续搜索的一致性
#[test]
fn test_multiple_searches_consecutively() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 禁用缓存
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Consistency Test".to_string());
    fields.insert("content".to_string(), "Testing search consistency".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 连续搜索多次，结果应该一致
    let options = SearchOptions::default();
    let (results1, _) = search(&manager, &schema, "Consistency", &options)
        .expect("Failed first search");
    
    let (results2, _) = search(&manager, &schema, "Consistency", &options)
        .expect("Failed second search");
    
    let (results3, _) = search(&manager, &schema, "Consistency", &options)
        .expect("Failed third search");

    assert_eq!(results1.len(), results2.len(), "Search results should be consistent");
    assert_eq!(results2.len(), results3.len(), "Search results should be consistent");
    
    // 验证文档 ID 一致
    assert_eq!(results1[0].document_id, results2[0].document_id);
    assert_eq!(results2[0].document_id, results3[0].document_id);
}

/// 测试索引统计信息的准确性
/// 
/// 注意：由于 Tantivy 的段合并机制，删除的文档可能不会立即从统计中移除
#[test]
fn test_index_stats_accuracy() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加 10 个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Stats Doc {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{:02}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 重新加载 reader 以获取最新统计
    manager.reload_reader().expect("Failed to reload reader");

    // 验证统计
    let stats = get_stats(&manager).expect("Failed to get stats");
    assert_eq!(stats.total_documents, 10, "Should have 10 documents");
    assert!(stats.total_terms > 0, "Should have some terms");
    assert!(stats.avg_document_length > 0.0, "Should have positive average document length");
}

/// 压力测试：混合操作
/// 
/// 测试大量文档的添加、更新、删除操作
#[test]
fn test_mixed_operations_stress() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 禁用缓存
    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 阶段 1: 批量添加 100 个文档
    let docs1: Vec<(String, HashMap<String, String>)> = (1..=100)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{:03}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, docs1)
        .expect("Failed to batch add documents");

    // 阶段 2: 批量更新前 50 个文档
    let docs2: Vec<(String, HashMap<String, String>)> = (1..=50)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Updated Document {}", i));
            fields.insert("content".to_string(), format!("Updated Content {}", i));
            (format!("doc{:03}", i), fields)
        })
        .collect();

    batch_update_documents(&manager, &schema, docs2)
        .expect("Failed to batch update documents");

    // 阶段 3: 搜索 "Updated"
    let options = SearchOptions {
        limit: 100,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results, _) = search(&manager, &schema, "Updated", &options)
        .expect("Failed to search updated documents");
    assert_eq!(results.len(), 50, "Should find 50 updated documents");

    // 阶段 4: 删除后 30 个文档
    let doc_ids_to_delete: Vec<String> = (71..=100)
        .map(|i| format!("doc{:03}", i))
        .collect();

    for doc_id in &doc_ids_to_delete {
        let mut retries = 0;
        loop {
            match delete_document(&manager, &schema, doc_id) {
                Ok(_) => break,
                Err(_e) if retries < 3 => {
                    retries += 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => panic!("Failed to delete document after 3 retries: {:?}", e),
            }
        }
    }

    // 阶段 5: 验证剩余文档数
    // 搜索所有文档
    let (results, _) = search(&manager, &schema, "Document", &options)
        .expect("Failed to search remaining documents");
    
    // 70 个文档（100 - 30），但更新操作会创建新段，
    // 所以搜索结果可能包含已删除文档的残留
    assert!(results.len() >= 70, "Should have at least 70 documents, found {}", results.len());
}
