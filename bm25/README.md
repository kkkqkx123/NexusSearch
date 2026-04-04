# BM25 Service

BM25 全文搜索服务和库，基于 Tantivy 搜索引擎实现。

## 功能特性

- 基于 BM25 算法的全文搜索
- 文档索引、更新、删除
- 字段加权
- 查询缓存
- 结果高亮
- gRPC 服务接口
- 嵌入式库模式

## 快速开始

### 作为库使用

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
bm25-service = { version = "0.1", default-features = false, features = ["embedded"] }
```

#### 基础示例

```rust
use bm25_service::{Bm25Index, Result};

fn main() -> Result<()> {
    // 创建索引
    let index = Bm25Index::create("/path/to/index")?;
    
    // 添加文档
    index.add_document("1", "Rust 编程", "Rust 是一门系统编程语言")?;
    index.add_document("2", "Java 编程", "Java 是一门面向对象语言")?;
    
    // 搜索
    let results = index.search("Rust", 10)?;
    for result in results {
        println!("{} - 得分：{:.4}", result.title.unwrap(), result.score);
    }
    
    Ok(())
}
```

#### 运行示例

```bash
# 运行基础搜索示例
cargo run --example basic_search

# 运行自定义配置示例
cargo run --example custom_config

# 运行 CRUD 操作示例
cargo run --example crud_operations
```

### 作为服务使用

#### 构建项目

```bash
cargo build --release --features service
```

#### 运行服务

```bash
cargo run --release --features service
```

#### 配置

服务支持通过环境变量或配置文件进行配置：

**环境变量**：
- `SERVER_ADDRESS`: 服务监听地址 (默认：0.0.0.0:50051)
- `REDIS_URL`: Redis 连接 URL (默认：redis://localhost:6379)
- `DATA_DIR`: 数据目录 (默认：./data)
- `INDEX_PATH`: 索引目录 (默认：./index)

**配置文件**：
编辑 `configs/config.toml` 文件进行配置。

## 使用模式

### 嵌入式模式（embedded）

适合将 BM25 搜索功能嵌入到现有应用中：

```toml
[dependencies]
bm25-service = { version = "0.1", default-features = false, features = ["embedded"] }
```

**特点**：
- 最小依赖
- 快速编译
- 简单 API

### 服务模式（service）

适合作为独立的微服务运行：

```toml
[dependencies]
bm25-service = { version = "0.1", features = ["service"] }
```

**特点**：
- gRPC 接口
- Redis 缓存
- 监控和日志

## API 文档

### Bm25Index 主要方法

- `create(path)` - 创建新索引
- `open(path)` - 打开已有索引
- `add_document(id, title, content)` - 添加文档
- `update_document(id, title, content)` - 更新文档
- `delete_document(id)` - 删除文档
- `search(query, limit)` - 搜索文档
- `count()` - 获取文档总数
- `commit()` - 提交更改

### IndexManagerConfig 配置项

```rust
use bm25_service::IndexManagerConfig;

let config = IndexManagerConfig::builder()
    .writer_memory_mb(50)      // 写入器内存预算
    .writer_threads(2)         // 写入器线程数
    .reader_cache(true)        // 启用 Reader 缓存
    .build();
```

## 开发

### 运行测试

```bash
cargo test
```

### 运行 Clippy

```bash
cargo clippy
```

### 格式化代码

```bash
cargo fmt
```

## gRPC 接口

服务提供以下 gRPC 接口：

- `IndexDocument`: 索引单个文档
- `BatchIndexDocuments`: 批量索引文档
- `Search`: 搜索文档
- `DeleteDocument`: 删除文档
- `GetStats`: 获取统计信息

## 技术栈

- Rust 2021
- Tantivy: 搜索引擎
- Tokio: 异步运行时
- Tonic: gRPC 框架（服务模式）
- Redis: 缓存和存储（服务模式）

## 特性说明

- `embedded` (默认): 嵌入式库模式，提供简化的 API
- `service`: 完整服务模式，包含 gRPC 服务器和缓存

**注意**：构建依赖已优化，库模式不会编译 proto 文件，加快编译速度。
