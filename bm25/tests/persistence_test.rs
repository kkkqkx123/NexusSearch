//! 持久化功能集成测试
//!
//! 测试备份、恢复、索引导入导出等功能

use bm25_service::api::core::{
    IndexManager, IndexSchema,
    persistence::{PersistenceManager, BackupInfo},
    batch::batch_add_documents,
};
use std::collections::HashMap;
use tempfile::TempDir;

#[test]
fn test_persistence_manager_creation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let manager = PersistenceManager::new(base_path);
    assert_eq!(manager.base_path(), base_path);
}

#[test]
fn test_create_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 创建备份
    let backup_info = persistence_manager.create_backup(&index_manager, "test_index")
        .expect("Failed to create backup");

    assert_eq!(backup_info.index_name, "test_index");
    assert!(backup_info.backup_path.exists());
}

#[test]
fn test_list_backups() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 创建多个备份
    let _ = persistence_manager.create_backup(&index_manager, "test_index")
        .expect("Failed to create first backup");

    // 短暂等待以确保时间戳不同
    std::thread::sleep(std::time::Duration::from_millis(100));

    let _ = persistence_manager.create_backup(&index_manager, "test_index")
        .expect("Failed to create second backup");

    // 列出备份
    let backups = persistence_manager.list_backups("test_index")
        .expect("Failed to list backups");

    assert!(backups.len() >= 2, "Should have at least 2 backups");

    // 验证备份按时间排序（最新的在前）
    assert!(backups[0].created_at >= backups[1].created_at);
}

#[test]
fn test_list_backups_empty() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let persistence_manager = PersistenceManager::new(base_path);

    let backups = persistence_manager.list_backups("nonexistent_index")
        .expect("Failed to list backups");

    assert!(backups.is_empty(), "Should have no backups for nonexistent index");
}

#[test]
fn test_delete_old_backups() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 创建 5 个备份
    for i in 0..5 {
        let _ = persistence_manager.create_backup(&index_manager, "test_index")
            .unwrap_or_else(|_| panic!("Failed to create backup {}", i));
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // 保留 2 个备份，删除其他
    let deleted = persistence_manager.delete_old_backups("test_index", 2)
        .expect("Failed to delete old backups");

    assert_eq!(deleted, 3, "Should delete 3 old backups");

    // 验证只剩下 2 个备份
    let backups = persistence_manager.list_backups("test_index")
        .expect("Failed to list backups");
    assert_eq!(backups.len(), 2, "Should have 2 backups remaining");
}

#[test]
fn test_delete_old_backups_keep_all() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 创建 3 个备份
    for i in 0..3 {
        let _ = persistence_manager.create_backup(&index_manager, "test_index")
            .unwrap_or_else(|_| panic!("Failed to create backup {}", i));
        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // 保留 5 个备份（超过实际数量）
    let deleted = persistence_manager.delete_old_backups("test_index", 5)
        .expect("Failed to delete old backups");

    assert_eq!(deleted, 0, "Should delete 0 backups when keep_count >= actual count");

    let backups = persistence_manager.list_backups("test_index")
        .expect("Failed to list backups");
    assert_eq!(backups.len(), 3, "Should still have 3 backups");
}

#[test]
fn test_get_index_metadata() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let persistence_manager = PersistenceManager::new(base_path);

    // 获取不存在的索引元数据
    let metadata = persistence_manager.get_index_metadata("nonexistent_index")
        .expect("Failed to get index metadata");

    assert_eq!(metadata.name, String::new());
    assert_eq!(metadata.path, String::new());
    assert_eq!(metadata.document_count, 0);
}

#[test]
fn test_get_index_size() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引
    let _ = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 获取索引大小
    let size = persistence_manager.get_index_size("test_index")
        .expect("Failed to get index size");

    assert!(size > 0, "Index size should be positive");
}

#[test]
fn test_get_index_size_nonexistent() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    let persistence_manager = PersistenceManager::new(base_path);

    // 获取不存在的索引大小
    let size = persistence_manager.get_index_size("nonexistent_index")
        .expect("Failed to get index size");

    assert_eq!(size, 0, "Nonexistent index should have size 0");
}

#[test]
fn test_compact_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 压缩索引
    let result = persistence_manager.compact_index(&index_manager);
    assert!(result.is_ok(), "Should successfully compact index");
}

#[test]
fn test_backup_info_serialization() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let backup_path = temp_dir.path().join("backup");

    let backup_info = BackupInfo {
        index_name: "test_index".to_string(),
        backup_id: "20240101_000000".to_string(),
        backup_path: backup_path.clone(),
        created_at: "2024-01-01T00:00:00Z".to_string(),
        size_bytes: 1024,
        document_count: 100,
    };

    // 测试 Clone
    let backup_info2 = backup_info.clone();
    assert_eq!(backup_info.index_name, backup_info2.index_name);

    // 测试 Debug
    let debug_str = format!("{:?}", backup_info);
    assert!(debug_str.contains("test_index"));
}

#[test]
fn test_export_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");
    let export_file = base_path.join("export.json");

    // 创建索引并添加文档
    let index_manager = IndexManager::create(&index_path)
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

    batch_add_documents(&index_manager, &schema, documents)
        .expect("Failed to add documents");

    let persistence_manager = PersistenceManager::new(base_path);

    // 导出索引
    let result = persistence_manager.export_index(&index_manager, "test_index", &export_file);
    assert!(result.is_ok(), "Should successfully export index");

    assert!(export_file.exists(), "Export file should exist");
}

#[test]
fn test_import_index() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");
    let import_file = base_path.join("import.json");

    // 创建导入文件
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create(&import_file).expect("Failed to create import file");
    writeln!(file, r#"{{"total_docs": 5}}"#).expect("Failed to write import file");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let persistence_manager = PersistenceManager::new(base_path);

    // 导入索引
    let result = persistence_manager.import_index(&index_manager, &schema, &import_file);
    assert!(result.is_ok(), "Should successfully import index");

    let count = result.expect("Failed to get import count");
    assert!(count > 0, "Import should add some data");
}

#[test]
fn test_restore_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");
    let restore_path = base_path.join("restored_index");

    // 创建索引
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");

    let persistence_manager = PersistenceManager::new(base_path);

    // 创建备份
    let backup_info = persistence_manager.create_backup(&index_manager, "test_index")
        .expect("Failed to create backup");

    // 恢复备份
    let result = persistence_manager.restore_backup("restored_index", &backup_info.backup_path);
    assert!(result.is_ok(), "Should successfully restore backup");

    assert!(restore_path.exists(), "Restored index should exist");
}

#[test]
fn test_multiple_indices_backup() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();

    // 创建多个索引
    let index1_path = base_path.join("index1");
    let index2_path = base_path.join("index2");

    let index1_manager = IndexManager::create(&index1_path)
        .expect("Failed to create first index");
    let index2_manager = IndexManager::create(&index2_path)
        .expect("Failed to create second index");

    let persistence_manager = PersistenceManager::new(base_path);

    // 为每个索引创建备份
    let _ = persistence_manager.create_backup(&index1_manager, "index1")
        .expect("Failed to create backup for index1");
    let _ = persistence_manager.create_backup(&index2_manager, "index2")
        .expect("Failed to create backup for index2");

    // 列出每个索引的备份
    let backups1 = persistence_manager.list_backups("index1")
        .expect("Failed to list backups for index1");
    let backups2 = persistence_manager.list_backups("index2")
        .expect("Failed to list backups for index2");

    assert!(!backups1.is_empty(), "Should have backups for index1");
    assert!(!backups2.is_empty(), "Should have backups for index2");
}

#[test]
fn test_backup_with_documents() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let base_path = temp_dir.path();
    let index_path = base_path.join("test_index");

    // 创建索引并添加文档
    let index_manager = IndexManager::create(&index_path)
        .expect("Failed to create index manager");
    let schema = IndexSchema::new();

    let documents: Vec<(String, HashMap<String, String>)> = (1..=10)
        .map(|i| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), format!("Document {}", i));
            fields.insert("content".to_string(), format!("Content for document {}", i));
            (format!("doc{}", i), fields)
        })
        .collect();

    batch_add_documents(&index_manager, &schema, documents)
        .expect("Failed to add documents");

    let persistence_manager = PersistenceManager::new(base_path);

    // 创建备份
    let backup_info = persistence_manager.create_backup(&index_manager, "test_index")
        .expect("Failed to create backup");

    assert!(backup_info.backup_path.exists());
    assert!(backup_info.size_bytes > 0);
}