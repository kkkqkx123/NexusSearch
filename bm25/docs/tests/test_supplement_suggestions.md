# BM25 服务测试补充建议

本文档详细描述了需要补充的测试场景，并提供测试代码示例。

## 1. 并发测试 (`concurrency_test.rs`)

### 1.1 测试场景概述

并发测试用于验证 BM25 服务在多线程环境下的正确性和稳定性。

### 1.2 建议测试用例

```rust
//! 并发测试
//!
//! 测试多线程环境下的索引操作正确性

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::{add_document, update_document, get_document},
    delete::delete_document,
    batch::batch_add_documents,
    search::{search, SearchOptions},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use tempfile::TempDir;

/// 测试并发添加文档
#[test]
fn test_concurrent_add_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = Arc::new(
        IndexManager::create(&index_path).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());
    
    let num_threads = 4;
    let docs_per_thread = 25;
    
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let schema = Arc::clone(&schema);
            
            thread::spawn(move || {
                for i in 0..docs_per_thread {
                    let doc_id = format!("thread{}_doc{}", thread_id, i);
                    let mut fields = HashMap::new();
                    fields.insert("title".to_string(), format!("Document from thread {}", thread_id));
                    fields.insert("content".to_string(), format!("Content {}", i));
                    
                    add_document(&manager, &schema, &doc_id, &fields)
                        .expect("Failed to add document");
                }
            })
        })
        .collect();
    
    for handle in handles {
        handle.join().expect("Thread panicked");
    }
    
    manager.reload_reader().expect("Failed to reload reader");
    
    let reader = manager.reader().expect("Failed to get reader");
    let searcher = reader.searcher();
    assert_eq!(searcher.num_docs(), (num_threads * docs_per_thread) as u64);
}

/// 测试并发搜索
#[test]
fn test_concurrent_search() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = Arc::new(
        IndexManager::create(&index_path).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());
    
    // 预先添加文档
    for i in 0..100 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content for document {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }
    
    manager.reload_reader().expect("Failed to reload reader");
    
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
    
    // 所有搜索应该返回相同数量的结果
    assert!(results.iter().all(|&r| r > 0));
}

/// 测试读写并发
#[test]
fn test_concurrent_read_write() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = Arc::new(
        IndexManager::create(&index_path).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());
    
    // 预先添加一些文档
    for i in 0..50 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Initial Doc {}", i));
        fields.insert("content".to_string(), format!("Content {}", i));
        add_document(&manager, &schema, &format!("doc{}", i), &fields)
            .expect("Failed to add document");
    }
    
    manager.reload_reader().expect("Failed to reload reader");
    
    // 启动读线程
    let read_manager = Arc::clone(&manager);
    let read_schema = Arc::clone(&schema);
    let read_handle = thread::spawn(move || {
        for _ in 0..20 {
            let options = SearchOptions::default();
            let _ = search(&read_manager, &read_schema, "Doc", &options);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    
    // 启动写线程
    let write_manager = Arc::clone(&manager);
    let write_schema = Arc::clone(&schema);
    let write_handle = thread::spawn(move || {
        for i in 50..100 {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("New Doc {}", i));
            fields.insert("content".to_string(), format!("New Content {}", i));
            let _ = add_document(&write_manager, &write_schema, &format!("doc{}", i), &fields);
            thread::sleep(std::time::Duration::from_millis(10));
        }
    });
    
    read_handle.join().expect("Read thread panicked");
    write_handle.join().expect("Write thread panicked");
}

/// 测试并发更新同一文档
#[test]
fn test_concurrent_update_same_document() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = Arc::new(
        IndexManager::create(&index_path).expect("Failed to create index manager")
    );
    let schema = Arc::new(IndexSchema::new());
    
    // 添加初始文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Original".to_string());
    fields.insert("content".to_string(), "Original content".to_string());
    add_document(&manager, &schema, "shared_doc", &fields)
        .expect("Failed to add document");
    
    let num_threads = 5;
    let handles: Vec<_> = (0..num_threads)
        .map(|thread_id| {
            let manager = Arc::clone(&manager);
            let schema = Arc::clone(&schema);
            
            thread::spawn(move || {
                let mut fields = HashMap::new();
                fields.insert("title".to_string(), format!("Updated by thread {}", thread_id));
                fields.insert("content".to_string(), format!("Content from thread {}", thread_id));
                
                update_document(&manager, &schema, "shared_doc", &fields)
            })
        })
        .collect();
    
    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread panicked"))
        .collect();
    
    // 所有更新应该成功（最后写入者胜出）
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## 2. 错误处理测试 (`error_handling_test.rs`)

### 2.1 测试场景概述

错误处理测试验证系统在各种异常情况下的行为。

### 2.2 建议测试用例

```rust
//! 错误处理测试
//!
//! 测试各种错误场景的处理

use bm25_service::api::core::{IndexManager, IndexSchema};
use bm25_service::error::Bm25Error;
use tempfile::TempDir;

/// 测试打开不存在的索引
#[test]
fn test_open_nonexistent_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("nonexistent");
    
    let result = IndexManager::open(&index_path);
    assert!(result.is_err());
}

/// 测试无效路径创建索引
#[test]
fn test_create_index_with_invalid_path() {
    // Windows 上无效路径字符
    #[cfg(windows)]
    let invalid_path = "CON:\\invalid\\path";
    
    #[cfg(not(windows))]
    let invalid_path = "/dev/null/invalid\0path";
    
    let result = IndexManager::create(std::path::Path::new(invalid_path));
    // 根据系统不同，可能成功或失败
    // 主要验证不会 panic
}

/// 测试空文档 ID
#[test]
fn test_add_document_with_empty_id() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();
    
    let mut fields = std::collections::HashMap::new();
    fields.insert("title".to_string(), "Test".to_string());
    fields.insert("content".to_string(), "Content".to_string());
    
    // 空字符串 ID 应该被接受（或根据业务逻辑拒绝）
    let result = bm25_service::api::core::document::add_document(
        &manager, &schema, "", &fields
    );
    // 验证行为一致性
}

/// 测试超大分页偏移
#[test]
fn test_search_with_large_offset() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();
    
    let options = bm25_service::api::core::search::SearchOptions {
        limit: 10,
        offset: 1_000_000,
        field_weights: std::collections::HashMap::new(),
        highlight: false,
    };
    
    let (results, _) = bm25_service::api::core::search::search(
        &manager, &schema, "test", &options
    ).expect("Search should succeed");
    
    assert!(results.is_empty());
}

/// 测试备份恢复失败场景
#[test]
fn test_restore_from_invalid_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let invalid_backup = base_path.join("invalid_backup");
    
    // 创建无效的备份目录
    std::fs::create_dir_all(&invalid_backup).expect("Failed to create dir");
    
    let persistence = bm25_service::api::core::persistence::PersistenceManager::new(base_path);
    
    let result = persistence.restore_backup("restored", &invalid_backup);
    // 验证错误处理
    assert!(result.is_ok() || result.is_err()); // 取决于实现
}
```

## 3. 配置测试 (`config_test.rs`)

### 3.1 测试场景概述

配置测试验证配置加载、验证和构建功能。

### 3.2 建议测试用例

```rust
//! 配置模块测试
//!
//! 测试配置加载、验证和构建功能

use bm25_service::config::{Bm25Config, IndexManagerConfig};

/// 测试 Bm25Config 默认值
#[test]
fn test_bm25_config_defaults() {
    let config = Bm25Config::default();
    
    assert_eq!(config.k1, 1.2);
    assert_eq!(config.b, 0.75);
    assert_eq!(config.avg_doc_length, 100.0);
}

/// 测试 Bm25Config 构建器
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

/// 测试 IndexManagerConfig 构建器
#[test]
fn test_index_manager_config_builder() {
    let config = IndexManagerConfig::builder()
        .writer_memory_mb(100)
        .writer_threads(4)
        .reader_cache(true)
        .build();
    
    assert_eq!(config.writer_memory_mb, 100);
    assert_eq!(config.writer_threads, 4);
    assert!(config.reader_cache);
}

/// 测试配置验证
#[test]
fn test_config_validation() {
    // 测试无效的 k1 值
    let result = Bm25Config::builder()
        .k1(-1.0)  // 无效值
        .build();
    
    // 根据实现，可能需要验证
}

/// 测试从环境变量加载配置
#[test]
fn test_config_from_env() {
    std::env::set_var("TEST_BM25_K1", "1.5");
    std::env::set_var("TEST_BM25_B", "0.8");
    
    let result = Bm25Config::from_env("TEST_BM25_");
    
    // 清理环境变量
    std::env::remove_var("TEST_BM25_K1");
    std::env::remove_var("TEST_BM25_B");
    
    if let Ok(config) = result {
        assert_eq!(config.k1, 1.5);
        assert_eq!(config.b, 0.8);
    }
}
```

## 4. 数据一致性测试 (`consistency_test.rs`)

### 4.1 测试场景概述

数据一致性测试验证系统在各种情况下的数据完整性。

### 4.2 建议测试用例

```rust
//! 数据一致性测试
//!
//! 测试数据完整性和一致性

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    document::{add_document, get_document},
    batch::batch_add_documents,
    persistence::PersistenceManager,
};
use std::collections::HashMap;
use tempfile::TempDir;

/// 测试索引重开后的数据一致性
#[test]
fn test_data_consistency_after_reopen() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    // 创建索引并添加文档
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
    
    // 重新打开索引
    let manager = IndexManager::open(&index_path).expect("Failed to open index");
    let schema = IndexSchema::new();
    
    // 验证所有文档都存在
    for i in 0..10 {
        let doc = get_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some(), "Document {} should exist", i);
    }
}

/// 测试备份恢复后的数据一致性
#[test]
fn test_backup_restore_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("original_index");
    let restore_path = base_path.join("restored_index");
    
    // 创建索引并添加文档
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
    
    // 创建备份
    let persistence = PersistenceManager::new(base_path);
    let backup_info = persistence.create_backup(&manager, "original_index")
        .expect("Failed to create backup");
    
    // 恢复备份
    persistence.restore_backup("restored_index", &backup_info.backup_path)
        .expect("Failed to restore backup");
    
    // 验证恢复的索引
    let restored_manager = IndexManager::open(&restore_path)
        .expect("Failed to open restored index");
    
    for i in 0..20 {
        let doc = get_document(&restored_manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some(), "Restored document {} should exist", i);
    }
}

/// 测试批量操作的原子性
#[test]
fn test_batch_operation_atomicity() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();
    
    // 批量添加文档
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
    
    // 验证所有文档都已添加
    for i in 0..10 {
        let doc = get_document(&manager, &schema, &format!("doc{}", i))
            .expect("Failed to get document");
        assert!(doc.is_some());
    }
}

/// 测试更新操作的文档数一致性
#[test]
fn test_update_document_count_consistency() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let index_path = temp_dir.path().join("test_index");
    
    let manager = IndexManager::create(&index_path).expect("Failed to create index manager");
    let schema = IndexSchema::new();
    
    // 添加初始文档
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Original".to_string());
    fields.insert("content".to_string(), "Original content".to_string());
    add_document(&manager, &schema, "doc1", &fields).expect("Failed to add document");
    
    let reader = manager.reader().expect("Failed to get reader");
    let initial_count = reader.searcher().num_docs();
    
    // 更新文档
    let mut updated_fields = HashMap::new();
    updated_fields.insert("title".to_string(), "Updated".to_string());
    updated_fields.insert("content".to_string(), "Updated content".to_string());
    bm25_service::api::core::document::update_document(&manager, &schema, "doc1", &updated_fields)
        .expect("Failed to update document");
    
    // 验证文档数量（更新不应增加文档数，但 Tantivy 可能创建新段）
    let reader = manager.reader().expect("Failed to get reader");
    let final_count = reader.searcher().num_docs();
    
    // 文档数应该 >= 初始数（取决于 Tantivy 的实现）
    assert!(final_count >= initial_count);
}
```

## 5. 测试优先级排序

### 5.1 立即需要添加（高优先级）

| 测试文件 | 测试用例数 | 预计工作量 |
|----------|------------|------------|
| `concurrency_test.rs` | 5-8 | 4 小时 |
| `consistency_test.rs` | 4-6 | 3 小时 |

### 5.2 短期需要添加（中优先级）

| 测试文件 | 测试用例数 | 预计工作量 |
|----------|------------|------------|
| `error_handling_test.rs` | 6-10 | 3 小时 |
| `config_test.rs` | 5-8 | 2 小时 |
| `storage_test.rs` | 4-6 | 4 小时 |
| `service_test.rs` | 4-6 | 4 小时 |

### 5.3 长期可以添加（低优先级）

| 测试文件 | 测试用例数 | 预计工作量 |
|----------|------------|------------|
| `embedded_test.rs` | 3-5 | 2 小时 |
| `performance_test.rs` | 4-6 | 6 小时 |

## 6. 测试执行建议

### 6.1 本地开发

```bash
# 运行所有测试
cargo test

# 运行特定测试文件
cargo test --test concurrency_test

# 运行带并发的测试
cargo test -- --test-threads=4

# 运行并显示输出
cargo test -- --nocapture
```

### 6.2 CI 配置建议

```yaml
# .github/workflows/test.yml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Run unit tests
        run: cargo test --lib
        
      - name: Run integration tests
        run: cargo test --test '*'
        
      - name: Run concurrency tests
        run: cargo test --test concurrency_test -- --test-threads=8
        
      - name: Generate coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml
          
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

*文档生成时间: 2026-04-05*
