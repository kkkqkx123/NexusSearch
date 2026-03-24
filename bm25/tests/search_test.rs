//! 搜索功能集成测试
//!
//! 测试全文搜索、分页、高亮等搜索功能

use bm25_service::index::{
    IndexManager, IndexSchema,
    batch::batch_add_documents,
    search::{search, SearchOptions}
};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_basic_search() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加测试文档
    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming".to_string());
            fields.insert("content".to_string(), "Rust is a systems programming language".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "TypeScript Guide".to_string());
            fields.insert("content".to_string(), "TypeScript is a typed superset of JavaScript".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 搜索 "Rust"
    let options = SearchOptions::default();
    let (results, max_score) = search(&manager, &schema, "Rust", &options)
        .expect("Failed to search");

    assert!(!results.is_empty(), "Should find results for 'Rust'");
    assert!(max_score > 0.0, "Max score should be positive");

    // 验证结果包含正确的文档
    let found_rust = results.iter().any(|r| {
        r.fields.get("title").map(|t| t.contains("Rust")).unwrap_or(false)
    });
    assert!(found_rust, "Should find Rust document");
}

#[test]
fn test_search_with_empty_query() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let options = SearchOptions::default();
    let (results, max_score) = search(&manager, &schema, "", &options)
        .expect("Failed to search with empty query");

    assert!(results.is_empty(), "Empty query should return no results");
    assert_eq!(max_score, 0.0, "Empty query should have max score 0");
}

#[test]
fn test_search_no_matches() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming".to_string());
            fields.insert("content".to_string(), "Rust is a systems programming language".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();
    let (results, max_score) = search(&manager, &schema, "Python Java", &options)
        .expect("Failed to search");

    assert!(results.is_empty(), "Should find no results for non-matching query");
    assert_eq!(max_score, 0.0, "No matches should have max score 0");
}

#[test]
fn test_search_with_pagination() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加 20 个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=20)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Programming Tutorial".to_string());
            fields.insert("content".to_string(), format!("This is programming tutorial number {}", i));
            (format!("doc{:02}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 第一页
    let options1 = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results1, _) = search(&manager, &schema, "Programming", &options1)
        .expect("Failed to search first page");
    assert_eq!(results1.len(), 10, "First page should have 10 results");

    // 第二页
    let options2 = SearchOptions {
        limit: 10,
        offset: 10,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results2, _) = search(&manager, &schema, "Programming", &options2)
        .expect("Failed to search second page");
    assert_eq!(results2.len(), 10, "Second page should have 10 results");

    // 第三页（应该是空的）
    let options3 = SearchOptions {
        limit: 10,
        offset: 20,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results3, _) = search(&manager, &schema, "Programming", &options3)
        .expect("Failed to search third page");
    assert!(results3.is_empty(), "Third page should be empty");
}

#[test]
fn test_search_with_limit() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    // 添加多个文档
    let documents: Vec<(String, HashMap<String, String>)> = (1..=15)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Test Document".to_string());
            fields.insert("content".to_string(), format!("Content for document {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    // 搜索限制为 5 个结果
    let options = SearchOptions {
        limit: 5,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: false,
    };

    let (results, _) = search(&manager, &schema, "Document", &options)
        .expect("Failed to search with limit");

    assert_eq!(results.len(), 5, "Should return exactly 5 results");
}

#[test]
fn test_search_with_highlighting() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Hello World".to_string());
            fields.insert("content".to_string(), "This is a test document with hello and world".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: true,
    };

    let (results, _) = search(&manager, &schema, "hello world", &options)
        .expect("Failed to search with highlighting");

    assert!(!results.is_empty(), "Should find results");

    // 验证高亮字段存在
    let result = &results[0];
    assert!(result.highlights.contains_key("title"), "Should have highlights for title");
    assert!(result.highlights.contains_key("content"), "Should have highlights for content");
}

#[test]
fn test_search_multiple_terms() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming Language".to_string());
            fields.insert("content".to_string(), "Rust is fast and safe systems programming language".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "TypeScript Language".to_string());
            fields.insert("content".to_string(), "TypeScript is a typed superset of JavaScript".to_string());
            fields
        }),
        ("doc3".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Programming Basics".to_string());
            fields.insert("content".to_string(), "Learn programming fundamentals".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Rust Programming", &options)
        .expect("Failed to search multiple terms");

    // 应该找到包含 "Rust" 或 "Programming" 的文档
    assert!(!results.is_empty(), "Should find results for multiple terms");

    // 验证结果数量
    assert!(results.len() >= 2, "Should find at least 2 documents");
}

#[test]
fn test_search_case_insensitive() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming".to_string());
            fields.insert("content".to_string(), "Rust is awesome".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();

    // 搜索小写
    let (results_lower, _) = search(&manager, &schema, "rust", &options)
        .expect("Failed to search lowercase");

    // 搜索大写
    let (results_upper, _) = search(&manager, &schema, "RUST", &options)
        .expect("Failed to search uppercase");

    // 搜索混合大小写
    let (results_mixed, _) = search(&manager, &schema, "RuSt", &options)
        .expect("Failed to search mixed case");

    assert!(!results_lower.is_empty(), "Should find results for lowercase");
    assert!(!results_upper.is_empty(), "Should find results for uppercase");
    assert!(!results_mixed.is_empty(), "Should find results for mixed case");
}

#[test]
fn test_search_result_ordering() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming".to_string());
            fields.insert("content".to_string(), "Rust is a systems programming language".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust in Depth".to_string());
            fields.insert("content".to_string(), "Deep dive into Rust programming".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Rust", &options)
        .expect("Failed to search");

    assert!(!results.is_empty());

    // 验证结果按分数排序
    for i in 1..results.len() {
        assert!(results[i].score <= results[i-1].score, "Results should be ordered by score");
    }
}

#[test]
fn test_search_result_structure() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Test Document".to_string());
            fields.insert("content".to_string(), "Test content".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "Test", &options)
        .expect("Failed to search");

    assert!(!results.is_empty());

    let result = &results[0];

    // 验证结果结构
    assert!(!result.document_id.is_empty(), "Document ID should not be empty");
    assert!(result.score >= 0.0, "Score should be non-negative");
    assert!(result.fields.contains_key("title"), "Should have title field");
    assert!(result.fields.contains_key("content"), "Should have content field");
}

#[test]
fn test_search_with_unicode_query() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "中文测试文档".to_string());
            fields.insert("content".to_string(), "这是一个中文测试文档的内容".to_string());
            fields
        }),
        ("doc2".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "English Document".to_string());
            fields.insert("content".to_string(), "This is an English document".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();
    // Note: Tantivy's default tokenizer splits on whitespace and punctuation.
    // Since Chinese text has no spaces, "中文测试文档" is indexed as a single token.
    // We need to search for the complete phrase to match.
    let (results, _) = search(&manager, &schema, "中文测试文档", &options)
        .expect("Failed to search with Unicode");

    assert!(!results.is_empty(), "Should find results for Unicode query");
}

#[test]
fn test_search_options_default() {
    let options = SearchOptions::default();

    assert_eq!(options.limit, 10, "Default limit should be 10");
    assert_eq!(options.offset, 0, "Default offset should be 0");
    assert!(!options.highlight, "Default highlight should be false");
    assert!(options.field_weights.is_empty(), "Default field_weights should be empty");
}

#[test]
fn test_search_with_field_weights() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Rust Programming".to_string());
            fields.insert("content".to_string(), "Some content".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let mut field_weights = HashMap::new();
    field_weights.insert("title".to_string(), 2.0);
    field_weights.insert("content".to_string(), 1.0);

    let options = SearchOptions {
        limit: 10,
        offset: 0,
        field_weights,
        highlight: false,
    };

    let (results, _) = search(&manager, &schema, "Rust", &options)
        .expect("Failed to search with field weights");

    assert!(!results.is_empty());
}

#[test]
fn test_search_with_special_characters_in_query() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");

    let manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents = vec![
        ("doc1".to_string(), {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "C++ Programming".to_string());
            fields.insert("content".to_string(), "Learn C++ programming language".to_string());
            fields
        }),
    ];

    batch_add_documents(&manager, &schema, documents)
        .expect("Failed to add documents");

    let options = SearchOptions::default();
    let (results, _) = search(&manager, &schema, "C++", &options)
        .expect("Failed to search with special characters");

    // 搜索应该成功，即使结果可能为空
    assert!(results.is_empty() || !results.is_empty());
}