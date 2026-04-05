# 条件编译体系修改方案

## 一、修改目标

1. 统一 Inversearch 和 BM25 的条件编译特性设计
2. 修正 Inversearch 特性依赖声明不准确的问题
3. 修复 service 模块无条件编译的问题
4. 为 BM25 添加存储抽象和可选 Redis 支持

## 二、Inversearch 修改方案

### 2.1 Cargo.toml 特性定义修改

**文件**: `inversearch/Cargo.toml`

```toml
[features]
default = ["embedded", "store"]
embedded = []
service = ["tonic", "prost", "tokio/full", "prost-build", "tonic-build"]
store = ["store-cold-warm-cache"]
store-cold-warm-cache = ["store-wal"]
store-file = []
store-redis = ["redis"]
store-wal = []
```

**变更说明**:
- `store-cold-warm-cache` 显式依赖 `store-wal`
- 保持 `store-redis` 独立，需要时单独启用

### 2.2 lib.rs service 模块条件编译修复

**文件**: `inversearch/src/lib.rs`

**问题**: `pub mod service;` 无条件编译，导致无论特性是否启用，模块都会参与编译。

**修改方案**:
```rust
#[cfg(feature = "service")]
pub mod proto;

#[cfg(feature = "service")]
pub mod service;

#[cfg(feature = "service")]
pub use api::server::{
    ServiceConfig, ServerConfig, run_server, InversearchService,
};

#[cfg(not(feature = "service"))]
pub mod service {
    pub use crate::api::embedded::EmbeddedIndex;
}
```

## 三、BM25 修改方案

### 3.1 Cargo.toml 特性定义修改

**文件**: `bm25/Cargo.toml`

```toml
[features]
default = ["embedded"]
embedded = []
service = [
    "tonic", "prost", "tokio/full",
    "prost-build", "tonic-build",
    "tracing", "tracing-subscriber", "metrics"
]
storage-redis = ["redis"]
```

**变更说明**:
- 移除 `service` 中的 `redis` 依赖，改为可选特性 `storage-redis`
- 使用标准库 `tracing` 相关依赖

### 3.2 为 BM25 添加存储抽象

**新增文件**: `bm25/src/storage/mod.rs`

提供与 Inversearch 一致的 `StorageInterface` trait 定义。

**修改 `bm25/src/lib.rs`**:

```rust
// Storage backends - conditionally available
#[cfg(feature = "storage-redis")]
pub mod redis;

#[cfg(feature = "storage-redis")]
pub use storage::redis::RedisStorage;
```

## 四、实施步骤

1. [x] 编写修改方案文档
2. [x] 修改 `inversearch/Cargo.toml` 特性定义 - 添加 `store-cold-warm-cache = ["store-wal"]` 依赖
3. [x] 验证 `inversearch/src/lib.rs` service 模块已有正确的条件编译
4. [x] 修改 `bm25/Cargo.toml` 特性定义 - 移除 `redis` 从 `service`，添加独立的 `storage-redis` 特性
5. [x] BM25 存储架构分析 - 确认 BM25 使用 Tantivy 内置存储，暂不添加额外存储抽象
6. [x] 验证修改后代码编译通过

## 五、实际修改内容

### 5.1 inversearch/Cargo.toml

```diff
  store-cold-warm-cache = []
+ store-cold-warm-cache = ["store-wal"]
```

### 5.2 bm25/Cargo.toml

```diff
  service = [
      "tonic",
      "prost",
      "tokio/full",
-     "redis",
      "tracing",
      "tracing-subscriber",
      "metrics",
      "prost-build",
      "tonic-build",
  ]
+ storage-redis = ["redis"]
```

## 六、验证结果

- ✅ inversearch: `cargo check` 通过
- ✅ bm25: `cargo check` 通过

## 七、向后兼容性

- 默认特性保持不变 (`default = ["embedded"]` 或 `default = ["embedded", "store"]`)
- 现有代码无需修改即可正常编译
- 可选特性需要用户显式启用