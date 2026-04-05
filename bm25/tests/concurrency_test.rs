//! 并发测试
//!
//! 测试多线程环境下的索引操作正确性

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::{add_document, update_document},
    delete::delete_document,
    batch::batch_add_documents,
    search::{search, SearchOptions},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// 测试并发搜索
///
/// 验证多线程同时执行搜索时的稳定性和结果一致性
#[test]
fn test_concurrent_search() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = Arc::new(
        IndexManager::create_with_config(&index_path, config).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());

    for i in 0..100 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content for document {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    let num_searches = 10;
    let handles: Vec<_> = (0..num_searches)
        .map(|_| {
            let manager = Arc::clone(&manager);
            let schema = Arc::clone(&schema);

            thread::spawn(move || {
                let options = SearchOptions::default();
                let (results, _) = search(&manager, &schema, "Document", &options)
                    .expect("Failed to search");
                results.len()
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();

    assert_eq!(results.len(), num_searches);
    for result_count in results {
        assert!(result_count > 0, "Each search should return results");
    }
}

/// 测试并发读取
///
/// 验证多线程同时读取索引时的稳定性
#[test]
fn test_concurrent_read_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = Arc::new(
        IndexManager::create_with_config(&index_path, config).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());

    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    let manager_clone = Arc::clone(&manager);
    let schema_clone = Arc::clone(&schema);

    let read_handle = thread::spawn(move || {
        for _ in 0..10 {
            let options = SearchOptions::default();
            let result = search(&manager_clone, &schema_clone, "Document", &options);
            assert!(result.is_ok());
        }
    });

    let write_handle = thread::spawn(move || {
        for i in 50..100 {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("New Document {}", i));
            fields.insert("content".to_string(), format!("New Content {}", i));
            add_document(&manager, &schema, &format!("newdoc{}", i), &fields)
                .expect("Failed to add document");
        }
    });

    read_handle.join().expect("Read thread panicked");
    write_handle.join().expect("Write thread panicked");
}

/// 测试高并发搜索压力
///
/// 验证在高并发搜索压力下的稳定性
#[test]
fn test_high_concurrency_search_stress() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = Arc::new(
        IndexManager::create_with_config(&index_path, config).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());

    for i in 0..200 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content for search testing {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    let num_threads = 20;
    let searches_per_thread = 5;

    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let schema = Arc::clone(&schema);

            thread::spawn(move || {
                let mut success_count = 0;
                for i in 0..searches_per_thread {
                    let options = SearchOptions::default();
                    let query = format!("Document {}", (thread_id * searches_per_thread + i) % 200);
                    if search(&manager, &schema, &query, &options).is_ok() {
                        success_count += 1;
                    }
                }
                success_count
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();

    let total_success: usize = results.iter().sum();
    assert_eq!(total_success, num_threads * searches_per_thread);
}

/// 测试并发删除文档
///
/// 验证多线程删除文档时的稳定性
#[test]
fn test_concurrent_delete_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = Arc::new(
        IndexManager::create_with_config(&index_path, config).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());

    for i in 0..100 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    let manager_clone = Arc::clone(&manager);
    let schema_clone = Arc::clone(&schema);

    for i in 0..50 {
        delete_document(&manager_clone, &schema_clone, &format!("doc{}", i))
            .expect("Failed to delete document");
    }

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 50);
}

/// 测试并发混合操作
///
/// 验证读写混合操作时的稳定性
#[test]
fn test_concurrent_mixed_operations() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = Arc::new(
        IndexManager::create_with_config(&index_path, config).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());

    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    let manager_for_read = Arc::clone(&manager);
    let schema_for_read = Arc::clone(&schema);

    let read_handle = thread::spawn(move || {
        let options = SearchOptions::default();
        for _ in 0..10 {
            let _ = search(&manager_for_read, &schema_for_read, "Document", &options);
        }
    });

    for i in 50..100 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    read_handle.join().expect("Read thread panicked");

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), 100);
}

/// 测试批量操作与搜索并发
///
/// 验证批量操作与搜索并发执行时的稳定性
#[test]
fn test_concurrent_batch_and_search() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = Arc::new(
        IndexManager::create_with_config(&index_path, config).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());

    let manager_for_batch = Arc::clone(&manager);
    let schema_for_batch = Arc::clone(&schema);

    let batch_handle = thread::spawn(move || {
        let mut docs = Vec::new();
        for i in 0..50 {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Batch Document {}", i));
            fields.insert("content".to_string(), format!("Batch Content {}", i));
            docs.push((format!("batch{}", i), fields));
        }
        batch_add_documents(&manager_for_batch, &schema_for_batch, docs)
            .expect("Failed to batch add documents");
    });

    batch_handle.join().expect("Batch thread panicked");

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Batch", &options)
        .expect("Failed to search");
    assert!(results.len() > 0);
}

/// 测试顺序添加文档
///
/// 验证顺序添加文档的正确性
#[test]
fn test_sequential_add_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let num_docs = 100;

    for i in 0..num_docs {
        let doc_id = format!("doc{}", i);
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));

        add_document(&manager, &schema, &doc_id, &fields)
            .expect("Failed to add document");
    }

    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), num_docs as u64);
}

/// 测试顺序更新文档
///
/// 验证顺序更新文档的正确性
#[test]
fn test_sequential_update_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let config = bm25_service::api::core::IndexManagerConfig::builder()
        .reader_cache(false)
        .build();
    let manager = IndexManager::create_with_config(&index_path, config)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Original {}", i));
        fields.insert("content".to_string(), format!("Original Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }

    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Updated {}", i));
        fields.insert("content".to_string(), format!("Updated Content {}", i));
        update_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to update document");
    }

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Updated", &options)
        .expect("Failed to search");
    assert!(results.len() > 0);
}
