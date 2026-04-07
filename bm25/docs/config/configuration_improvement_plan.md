# BM25 服务配置模块改进方案

## 概述

本文档详细说明了 BM25 服务配置模块的改进方案，基于对当前配置文件 `configs/config.toml` 与代码实现的分析。

**当前状态**：配置系统整体完善，支持文件加载、环境变量、构建器模式和验证机制。

**匹配度评分**：90% ⭐⭐⭐⭐

---

## 一、发现的问题

### 1.1 StorageConfig 嵌套结构问题

**问题描述**：

配置文件使用了嵌套的 TOML 结构：

```toml
[storage]
type = "tantivy"

[storage.tantivy]
index_path = "./index"
writer_memory_mb = 50

[storage.redis]
url = "redis://127.0.0.1:6379"
pool_size = 10
```

但代码中的 `StorageConfig` 结构可能不支持这种嵌套解析方式。

**当前代码结构**：

```rust
// src/config/mod.rs
pub struct StorageConfig {
    pub storage_type: StorageType,
    pub tantivy: Option<TantivyStorageConfig>,
    pub redis: Option<RedisStorageConfig>,
}

pub enum StorageType {
    Tantivy,
    Redis,
}
```

**潜在问题**：
- TOML 解析时，`storage.type` 和 `storage.tantivy.*` 可能无法正确映射到嵌套结构
- 需要验证 `serde` 的嵌套反序列化是否正常工作

### 1.2 缺少配置热重载支持

**问题描述**：

当前配置只在服务启动时加载一次，运行时无法更新配置。

**影响**：
- 修改配置后需要重启服务
- 不适合需要动态调整配置的场景（如调整 BM25 参数、合并策略等）

### 1.3 环境变量覆盖逻辑分散

**问题描述**：

环境变量加载逻辑分散在多个地方：
- `Config::from_env()` 中有硬编码的环境变量读取
- `EnvLoader` 提供了通用的环境变量加载器，但使用不一致

---

## 二、改进方案

### 2.1 方案 A：保持嵌套结构（推荐）

**目标**：支持配置文件中的嵌套 TOML 结构

**实现步骤**：

#### 步骤 1：确认 StorageConfig 结构

```rust
// src/config/mod.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: StorageType,
    
    #[serde(default)]
    pub tantivy: Option<TantivyStorageConfig>,
    
    #[serde(default)]
    pub redis: Option<RedisStorageConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TantivyStorageConfig {
    pub index_path: String,
    pub writer_memory_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisStorageConfig {
    pub url: String,
    pub pool_size: u32,
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,
    #[serde(default)]
    pub key_prefix: String,
    #[serde(default)]
    pub min_idle: u32,
    #[serde(default)]
    pub max_lifetime_secs: u64,
}

fn default_connection_timeout() -> u64 { 5 }
```

#### 步骤 2：添加测试验证嵌套解析

```rust
// tests/config_test.rs
#[test]
fn test_storage_config_nested_toml() {
    let toml_content = r#"
        [storage]
        type = "tantivy"
        
        [storage.tantivy]
        index_path = "./index"
        writer_memory_mb = 50
        
        [storage.redis]
        url = "redis://localhost:6379"
        pool_size = 10
    "#;
    
    let config: Config = toml::from_str(toml_content).unwrap();
    
    assert_eq!(config.storage.storage_type, StorageType::Tantivy);
    assert!(config.storage.tantivy.is_some());
    assert_eq!(config.storage.tantivy.as_ref().unwrap().index_path, "./index");
}
```

#### 步骤 3：如果嵌套解析失败，提供备选方案

**备选方案 A**：使用扁平化 TOML 结构

```toml
# 备选配置文件格式
[storage]
type = "tantivy"
tantivy_index_path = "./index"
tantivy_writer_memory_mb = 50
redis_url = "redis://localhost:6379"
redis_pool_size = 10
```

**备选方案 B**：使用 `flatten` 属性

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    #[serde(rename = "type")]
    pub storage_type: StorageType,
    
    #[serde(flatten)]
    pub tantivy: Option<TantivyStorageConfig>,
    
    #[serde(flatten)]
    pub redis: Option<RedisStorageConfig>,
}
```

---

### 2.2 方案 B：配置热重载支持

**目标**：支持运行时动态更新配置

**实现步骤**：

#### 步骤 1：添加配置监听器

```rust
// src/config/watcher.rs
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use tokio::sync::broadcast;

pub struct ConfigWatcher {
    config_path: String,
    tx: broadcast::Sender<Config>,
}

impl ConfigWatcher {
    pub fn new(config_path: &str, tx: broadcast::Sender<Config>) -> Self {
        Self {
            config_path: config_path.to_string(),
            tx,
        }
    }
    
    pub fn start(&mut self) -> anyhow::Result<()> {
        let (tx, mut rx) = broadcast::channel(16);
        let config_path = self.config_path.clone();
        
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(event) = res {
                    if event.kind.is_modify() {
                        // 重新加载配置
                        if let Ok(new_config) = Config::from_file(&config_path) {
                            tx.send(new_config).ok();
                        }
                    }
                }
            },
            Config::default(),
        )?;
        
        watcher.watch(Path::new(&config_path), RecursiveMode::NonRecursive)?;
        
        Ok(())
    }
}
```

#### 步骤 2：在服务器中集成配置热重载

```rust
// src/api/server/mod.rs
use tokio::sync::broadcast;
use std::sync::Arc;
use tokio::sync::RwLock;

pub async fn run_server(config: Config) -> anyhow::Result<()> {
    let config = Arc::new(RwLock::new(config));
    let (tx, mut rx) = broadcast::channel(16);
    
    // 启动配置监听器
    let config_for_watcher = config.clone();
    tokio::spawn(async move {
        while let Ok(new_config) = rx.recv().await {
            let mut config = config_for_watcher.write().await;
            *config = new_config;
            tracing::info!("Configuration reloaded");
        }
    });
    
    // 使用 config 运行服务器
    // ...
    
    Ok(())
}
```

#### 步骤 3：添加热重载 API

```rust
// src/api/grpc/mod.rs
async fn reload_config(
    &self,
    _request: Request<ReloadConfigRequest>,
) -> Result<Response<ReloadConfigResponse>, Status> {
    // 触发配置重载
    // 可以通过文件监听或手动触发
    
    Ok(Response::new(ReloadConfigResponse {
        success: true,
        message: "Configuration reloaded".to_string(),
    }))
}
```

---

### 2.3 方案 C：统一环境变量加载逻辑

**目标**：统一使用 `EnvLoader` 和 `apply_vars` 模式

#### 步骤 1：重构 Config::from_env()

```rust
// src/api/server/config.rs
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let mut config = Self::default();
        
        // 使用统一的 EnvLoader
        let loader = EnvLoader::new(""); // 无前缀，读取所有环境变量
        let vars = loader.load()?;
        
        // 应用环境变量
        config.server.apply_vars(&vars)?;
        config.index.apply_vars(&vars)?;
        config.bm25.apply_vars(&vars)?;
        config.search.apply_vars(&vars)?;
        config.storage.apply_vars(&vars)?;
        
        config.validate()?;
        Ok(config)
    }
}
```

#### 步骤 2：为每个配置结构实现 apply_vars

```rust
// src/config/mod.rs
impl ServerConfig {
    fn apply_vars(&mut self, vars: &HashMap<String, String>) -> Result<(), LoaderError> {
        if let Some(val) = vars.get("server.address") {
            self.address = val.parse()
                .map_err(|e| LoaderError::ParseError(format!("server.address: {}", e)))?;
        }
        Ok(())
    }
}
```

---

## 三、实施计划

### 阶段 1：验证嵌套 TOML 支持（1 天）

- [ ] 编写测试验证当前嵌套 TOML 是否能正确解析
- [ ] 如果失败，确定使用扁平化还是修改结构体

### 阶段 2：配置热重载（2-3 天）

- [ ] 添加 `notify` 依赖
- [ ] 实现 `ConfigWatcher`
- [ ] 集成到服务器
- [ ] 添加 gRPC 重载 API
- [ ] 编写测试

### 阶段 3：统一环境变量逻辑（1 天）

- [ ] 重构 `Config::from_env()`
- [ ] 为所有配置结构实现 `apply_vars()`
- [ ] 更新文档

### 阶段 4：文档和示例（1 天）

- [ ] 更新配置示例
- [ ] 添加配置项说明
- [ ] 编写配置热重载使用指南

---

## 四、依赖更新

```toml
# Cargo.toml
[dependencies]
# 添加配置热重载支持
notify = "6.1"
```

---

## 五、测试计划

### 5.1 单元测试

```rust
#[test]
fn test_config_nested_toml_parsing()
#[test]
fn test_config_hot_reload()
#[test]
fn test_env_var_override()
```

### 5.2 集成测试

```rust
#[test]
fn test_config_file_loading()
#[test]
fn test_config_validation()
#[test]
fn test_config_merge_policy()
```

---

## 六、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|----------|
| 嵌套 TOML 解析失败 | 中 | 低 | 使用扁平化备选方案 |
| 热重载导致状态不一致 | 高 | 中 | 限制可热重载的配置项 |
| 环境变量命名冲突 | 低 | 低 | 使用明确的前缀和命名 |

---

## 七、总结

**优先级**：
1. ✅ **高优先级**：验证嵌套 TOML 支持
2. 🔶 **中优先级**：统一环境变量逻辑
3. 🔷 **低优先级**：配置热重载（按需实现）

**预期收益**：
- 提高配置系统的可靠性和易用性
- 支持动态配置更新，减少服务重启
- 统一的配置加载逻辑，便于维护

---

**创建时间**：2026-04-07  
**作者**：AI Assistant  
**版本**：1.0
