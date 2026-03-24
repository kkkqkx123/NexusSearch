//! 综合集成测试
//!
//! 测试完整的索引生命周期，包括文档管理、搜索、持久化等功能的组合使用

use bm25_service::index::{
    IndexManager, IndexSchema,
    document::{add_document, update_document, get_document},
    batch::{batch_add_documents, batch_update_documents},
    delete::delete_document,
    search::{search, SearchOptions},
    persistence::PersistenceManager,
    cache::Cache,
    stats::get_stats,
};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_full_index_lifecycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    // 1. 创建索引
    let manager = IndexManager::create(&index_path)
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

    // 5. 搜索文档
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

#[test]
fn test_batch_operations_with_search() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
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
    assert_eq!(results_page1.len(), 10);

    let options_page2 = SearchOptions {
        limit: 10,
        offset: 10,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results_page2, _) = search(&manager, &schema, "Document", &options_page2)
        .expect("Failed to search page 2");
    assert_eq!(results_page2.len(), 10);
}

#[test]
fn test_search_with_update_and_delete() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加文档
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
fn test_cache_with_document_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 创建缓存
    let cache: Cache<String, String> = Cache::new(10, 60);

    // 添加文档并缓存
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Cached Document".to_string());
    fields.insert("content".to_string(), "Content to cache".to_string());

    add_document(&manager, &schema, "doc1", &fields)
        .expect("Failed to add document");

    // 缓存文档 ID
    cache.insert("doc1".to_string(), "cached_content".to_string());

    // 从缓存获取
    let cached = cache.get(&"doc1".to_string());
    assert_eq!(cached, Some("cached_content".to_string()));

    // 更新文档
    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated Cached Document".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());

    update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");

    // 更新缓存
    cache.insert("doc1".to_string(), "updated_cached_content".to_string());

    let cached = cache.get(&"doc1".to_string());
    assert_eq!(cached, Some("updated_cached_content".to_string()));

    // 删除文档和缓存
    delete_document(&manager, &schema, "doc1")
        .expect("Failed to delete document");
    cache.remove(&"doc1".to_string());

    let cached = cache.get(&"doc1".to_string());
    assert_eq!(cached, None);
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

    let manager = IndexManager::create(&index_path)
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

#[test]
fn test_mixed_operations_stress() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
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

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 100);

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
        delete_document(&manager, &schema, doc_id)
            .expect("Failed to delete document");
    }

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 70, "Should have 70 documents after deletion");

    // 阶段 5: 批量添加 30 个新文档
    let docs3: Vec<(String, HashMap<String, String>)> = (101..=130)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("New Document {}", i));
            fields.insert("content".to_string(), format!("New Content {}", i));
            (format!("doc{:03}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, docs3)
        .expect("Failed to batch add new documents");

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 100, "Should have 100 documents after adding new ones");
}

#[test]
fn test_search_with_highlights_and_pagination() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = (1..=15)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Test Document {}", i));
            fields.insert("content".to_string(), format!("This is test content number {} with search keywords", i));
            (format!("doc{:02}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 搜索并启用高亮
    let options = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: true,
    };

    let (results1, _) = search(&manager, &schema, "test", &options)
        .expect("Failed to search with highlights");
    assert_eq!(results1.len(), 10);

    // 验证高亮字段存在
    for result in &results1 {
        assert!(result.highlights.contains_key("title"));
        assert!(result.highlights.contains_key("content"));
    }

    // 第二页
    let options2 = SearchOptions {
        limit: 10,
        offset: 10,
        field_weights: HashMap::new(),
        highlight: true,
    };

    let (results2, _) = search(&manager, &schema, "test", &options2)
        .expect("Failed to search page 2");
    assert_eq!(results2.len(), 5);
}

#[test]
fn test_index_stats_accuracy() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 初始统计
    let stats1 = get_stats(&manager)
        .expect("Failed to get initial stats");
    assert_eq!(stats1.total_documents, 0);

    // 添加 10 个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Doc {}", i));
            fields.insert("content".to_string(), format!("Content {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 统计应该更新
    let stats2 = get_stats(&manager)
        .expect("Failed to get stats after adding");
    assert_eq!(stats2.total_documents, 10);

    // 删除 3 个文档
    delete_document(&manager, &schema, "doc1")
        .expect("Failed to delete doc1");
    delete_document(&manager, &schema, "doc2")
        .expect("Failed to delete doc2");
    delete_document(&manager, &schema, "doc3")
        .expect("Failed to delete doc3");

    // 统计应该更新
    let stats3 = get_stats(&manager)
        .expect("Failed to get stats after deletion");
    assert_eq!(stats3.total_documents, 7);
}

#[test]
fn test_multiple_searches_consecutively() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming".to_string());
            fields.insert("content".to_string(), "Rust is fast and safe".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "TypeScript Programming".to_string());
            fields.insert("content".to_string(), "TypeScript is typed JavaScript".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();

    // 连续执行多次搜索
    for _ in 0..5 {
        let (results, _) = search(&manager, &schema, "Programming", &options)
            .expect("Failed to search");
        assert_eq!(results.len(), 2);
    }

    for _ in 0..5 {
        let (results, _) = search(&manager, &schema, "Rust", &options)
            .expect("Failed to search");
        assert_eq!(results.len(), 1);
    }

    for _ in 0..5 {
        let (results, _) = search(&manager, &schema, "nonexistent", &options)
            .expect("Failed to search");
        assert_eq!(results.len(), 0);
    }
}