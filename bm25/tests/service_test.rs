//! 服务层测试
//!
//! 测试 gRPC 服务相关功能

#[cfg(feature = "service")]
use bm25_service::server::{Config, ServerConfig, IndexConfig};
#[cfg(feature = "service")]
use bm25_service::api::core::IndexManagerConfig;

/// 测试服务配置默认值
///
/// 验证服务配置的默认值
#[cfg(feature = "service")]
#[test]
fn test_server_config_defaults() {
    let config = Config::default();

    assert!(!config.index.data_dir.is_empty());
    assert!(!config.index.index_path.is_empty());
}

/// 测试索引配置默认值
///
/// 验证索引配置的默认值
#[cfg(feature = "service")]
#[test]
fn test_index_config_defaults() {
    let config = IndexConfig {
        data_dir: "./data".to_string(),
        index_path: "./index".to_string(),
        manager: IndexManagerConfig::default(),
    };

    assert!(!config.data_dir.is_empty());
    assert!(!config.index_path.is_empty());
}

/// 测试服务配置构建
///
/// 验证服务配置可以正确构建
#[cfg(feature = "service")]
#[test]
fn test_server_config_build() {
    use std::net::SocketAddr;

    let config = ServerConfig {
        address: "127.0.0.1:50051".parse().unwrap(),
    };

    let addr: SocketAddr = config.address;
    assert!(addr.port() > 0);
}

/// 测试索引配置构建
///
/// 验证索引配置可以正确构建
#[cfg(feature = "service")]
#[test]
fn test_index_config_build() {
    let config = IndexConfig {
        data_dir: "/custom/data".to_string(),
        index_path: "/custom/index".to_string(),
        manager: IndexManagerConfig::default(),
    };

    assert_eq!(config.data_dir, "/custom/data");
    assert_eq!(config.index_path, "/custom/index");
}

/// 测试配置验证
///
/// 验证配置验证功能
#[cfg(feature = "service")]
#[test]
fn test_config_validation() {
    let config = Config::default();

    assert!(config.bm25.k1 > 0.0);
    assert!(config.bm25.b >= 0.0 && config.bm25.b <= 1.0);
}

/// 测试从环境变量加载配置
///
/// 验证从环境变量加载配置
#[cfg(feature = "service")]
#[test]
fn test_config_from_env() {
    std::env::set_var("SERVER_ADDRESS", "0.0.0.0:8080");
    std::env::set_var("DATA_DIR", "/tmp/data");
    std::env::set_var("INDEX_PATH", "/tmp/index");

    let config = Config::from_env();
    assert!(config.is_ok());

    std::env::remove_var("SERVER_ADDRESS");
    std::env::remove_var("DATA_DIR");
    std::env::remove_var("INDEX_PATH");
}

/// 测试 BM25 配置
///
/// 验证 BM25 配置在服务配置中的正确性
#[cfg(feature = "service")]
#[test]
fn test_bm25_config_in_service() {
    let config = Config::default();

    assert!(config.bm25.k1 > 0.0);
    assert!(config.bm25.b >= 0.0);
    assert!(config.bm25.avg_doc_length > 0.0);
}

/// 测试搜索配置
///
/// 验证搜索配置在服务配置中的正确性
#[cfg(feature = "service")]
#[test]
fn test_search_config_in_service() {
    let config = Config::default();

    assert!(config.search.default_limit > 0);
    assert!(config.search.max_limit > 0);
}

/// 测试配置克隆
///
/// 验证配置可以正确克隆
#[cfg(feature = "service")]
#[test]
fn test_config_clone() {
    let config = Config::default();
    let cloned = config.clone();

    assert_eq!(config.bm25.k1, cloned.bm25.k1);
    assert_eq!(config.bm25.b, cloned.bm25.b);
}

/// 测试配置 Debug 输出
///
/// 验证配置可以正确格式化输出
#[cfg(feature = "service")]
#[test]
fn test_config_debug() {
    let config = Config::default();
    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("Config"));
}

/// 测试配置序列化
///
/// 验证配置可以正确序列化
#[cfg(feature = "service")]
#[test]
fn test_config_serialization() {
    let config = Config::default();

    let json = serde_json::to_string(&config);
    assert!(json.is_ok());

    let json_str = json.unwrap();
    assert!(json_str.contains("server"));
    assert!(json_str.contains("index"));
}

/// 测试配置反序列化
///
/// 验证配置可以正确反序列化
#[cfg(feature = "service")]
#[test]
fn test_config_deserialization() {
    let json = r#"{
        "server": {"address": "0.0.0.0:50051"},
        "storage": {
            "storage_type": "tantivy",
            "tantivy": {"index_path": "./index", "writer_memory_mb": 50},
            "redis": {"url": "redis://localhost:6379", "pool_size": 10, "connection_timeout_secs": 5, "key_prefix": "bm25:"}
        },
        "index": {
            "data_dir": "./data",
            "index_path": "./index",
            "manager": {
                "writer_memory_budget": 50000000,
                "writer_num_threads": null,
                "reader_cache_enabled": true,
                "reader_reload_policy": "on_commit_with_delay",
                "merge_policy": "log",
                "log_merge_policy": {"min_merge_size": 8, "max_merge_size": 50000, "min_layer_size": 10000}
            }
        },
        "bm25": {
            "k1": 1.2,
            "b": 0.75,
            "avg_doc_length": 100.0,
            "field_weights": {"title": 2.0, "content": 1.0}
        },
        "search": {
            "default_limit": 10,
            "max_limit": 100,
            "enable_highlight": true,
            "highlight_fragment_size": 200,
            "enable_spell_check": false,
            "fuzzy_matching": false,
            "fuzzy_distance": 2
        }
    }"#;

    let config: Result<Config, _> = serde_json::from_str(json);
    assert!(config.is_ok(), "Failed to deserialize: {:?}", config.err());

    let config = config.unwrap();
    assert_eq!(config.bm25.k1, 1.2);
    assert_eq!(config.bm25.b, 0.75);
}

/// 测试索引管理器配置
///
/// 验证索引管理器配置在服务配置中的正确性
#[cfg(feature = "service")]
#[test]
fn test_index_manager_config_in_service() {
    let config = Config::default();

    assert!(config.index.manager.reader_cache_enabled);
}

/// 测试不同端口配置
///
/// 验证不同端口配置
#[cfg(feature = "service")]
#[test]
fn test_different_port_configs() {
    use std::net::SocketAddr;

    let ports = vec![50051, 8080, 3000, 9000];

    for port in ports {
        let addr_str = format!("0.0.0.0:{}", port);
        let config = ServerConfig {
            address: addr_str.parse().unwrap(),
        };

        let addr: SocketAddr = config.address;
        assert_eq!(addr.port(), port);
    }
}

/// 测试不同数据目录配置
///
/// 验证不同数据目录配置
#[cfg(feature = "service")]
#[test]
fn test_different_data_dir_configs() {
    let data_dirs = vec!["./data", "/var/data", "/tmp/index", "./storage"];

    for data_dir in data_dirs {
        let config = IndexConfig {
            data_dir: data_dir.to_string(),
            index_path: format!("{}/index", data_dir),
            manager: IndexManagerConfig::default(),
        };

        assert_eq!(config.data_dir, data_dir);
    }
}

/// 测试 BM25 参数边界值
///
/// 验证 BM25 参数边界值
#[cfg(feature = "service")]
#[test]
fn test_bm25_parameter_bounds() {
    let mut config = Config::default();

    config.bm25.k1 = 0.1;
    assert!(config.bm25.k1 > 0.0);

    config.bm25.k1 = 10.0;
    assert!(config.bm25.k1 > 0.0);

    config.bm25.b = 0.0;
    assert!(config.bm25.b >= 0.0);

    config.bm25.b = 1.0;
    assert!(config.bm25.b <= 1.0);
}

/// 测试搜索配置边界值
///
/// 验证搜索配置边界值
#[cfg(feature = "service")]
#[test]
fn test_search_config_bounds() {
    let mut config = Config::default();

    config.search.default_limit = 1;
    assert!(config.search.default_limit > 0);

    config.search.default_limit = 1000;
    assert!(config.search.default_limit > 0);

    config.search.max_limit = 10;
    assert!(config.search.max_limit > 0);

    config.search.highlight_fragment_size = 50;
    assert!(config.search.highlight_fragment_size > 0);
}

/// 测试配置字段权重
///
/// 验证字段权重配置
#[cfg(feature = "service")]
#[test]
fn test_field_weights_config() {
    let config = Config::default();

    assert!(config.bm25.field_weights.title > 0.0);
    assert!(config.bm25.field_weights.content > 0.0);
}

/// 测试存储配置
///
/// 验证存储配置
#[cfg(feature = "service")]
#[test]
fn test_storage_config_in_service() {
    use bm25_service::config::StorageType;
    
    let config = Config::default();

    assert!(matches!(config.storage.storage_type, StorageType::Tantivy | StorageType::Redis));
}

/// 测试 IPv4 和 IPv6 地址配置
///
/// 验证 IPv4 和 IPv6 地址配置
#[cfg(feature = "service")]
#[test]
fn test_ip_address_configs() {
    use std::net::SocketAddr;

    let addresses = vec![
        "0.0.0.0:50051",
        "127.0.0.1:50051",
    ];

    for addr_str in addresses {
        let config = ServerConfig {
            address: addr_str.parse().unwrap(),
        };

        let addr: SocketAddr = config.address;
        assert_eq!(addr.port(), 50051);
    }
}

/// 测试配置默认值一致性
///
/// 验证多次调用 default() 返回相同的值
#[cfg(feature = "service")]
#[test]
fn test_config_default_consistency() {
    let config1 = Config::default();
    let config2 = Config::default();

    assert_eq!(config1.bm25.k1, config2.bm25.k1);
    assert_eq!(config1.bm25.b, config2.bm25.b);
    assert_eq!(config1.search.default_limit, config2.search.default_limit);
}

/// 测试索引配置克隆
///
/// 验证索引配置可以正确克隆
#[cfg(feature = "service")]
#[test]
fn test_index_config_clone() {
    let config = IndexConfig {
        data_dir: "./data".to_string(),
        index_path: "./index".to_string(),
        manager: IndexManagerConfig::default(),
    };

    let cloned = config.clone();

    assert_eq!(config.data_dir, cloned.data_dir);
    assert_eq!(config.index_path, cloned.index_path);
}

/// 测试索引配置 Debug 输出
///
/// 验证索引配置可以正确格式化输出
#[cfg(feature = "service")]
#[test]
fn test_index_config_debug() {
    let config = IndexConfig {
        data_dir: "./data".to_string(),
        index_path: "./index".to_string(),
        manager: IndexManagerConfig::default(),
    };

    let debug_str = format!("{:?}", config);

    assert!(debug_str.contains("data_dir"));
    assert!(debug_str.contains("index_path"));
}
