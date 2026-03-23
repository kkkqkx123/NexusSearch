# BM25 服务分阶段执行方案

## 概述

本文档基于 FlexSearch 中间层架构设计和 BM25 服务模块设计，提供详细的分阶段执行方案。方案将 BM25 服务开发划分为 5 个阶段，从基础设施搭建到功能完善，逐步实现完整的 BM25 全文搜索服务。

## 技术选型

### 核心依赖

| 依赖 | 版本 | 用途 | 说明 |
|------|------|------|------|
| **Tantivy** | 0.22+ | 搜索引擎核心 | 提供完整的全文搜索功能，包括 BM25 评分、倒排索引等 |
| **Tokio** | 1.35+ | 异步运行时 | 高性能异步 I/O 和任务调度 |
| **Tonic** | 0.11+ | gRPC 框架 | 服务间通信协议实现 |
| **Redis** | 0.24+ | 缓存和存储 | 查询缓存和索引持久化 |
| **Tracing** | 0.1+ | 日志系统 | 结构化日志和分布式追踪 |
| **Metrics** | 0.22+ | 监控指标 | 性能指标收集和导出 |
| **Serde** | 1.0+ | 序列化 | 数据序列化和反序列化 |
| **Prost** | 0.12+ | Protocol Buffers | gRPC 消息编解码 |

### Tantivy 库说明

**选择理由**：
- Tantivy 是 Rust 生态中最成熟的全文搜索引擎库，受 Apache Lucene 启发
- 内置 BM25 评分算法，无需自行实现复杂的评分逻辑
- 提供完整的倒排索引、词频统计、文档频率统计等功能
- 支持字段加权、高亮显示、同义词扩展等高级特性
- 高性能，支持 SIMD 优化和并发索引

**核心特性**：
- `Bm25StatisticsProvider` trait：提供 BM25 评分所需的统计信息
- `FieldNormReader`：字段长度归一化，优化评分计算
- `BlockSegmentPostings`：块级最大分数计算，支持高性能场景
- 支持自定义评分函数和权重配置

---

## 阶段一：基础设施搭建（Week 1）

### 目标

搭建 BM25 服务的基础架构，包括项目结构、配置管理、gRPC 服务框架等。

### 任务清单

#### 1.1 项目初始化

**任务描述**：创建 Rust 项目结构，配置 Cargo.toml

**具体步骤**：
1. 创建 `services/bm25` 目录
2. 初始化 Cargo 项目：`cargo init`
3. 配置 `Cargo.toml` 依赖
4. 创建基础目录结构

**产出物**：
- `services/bm25/Cargo.toml`
- `services/bm25/src/` 目录结构
- `services/bm25/proto/` 目录

**代码示例**：

```toml
[package]
name = "bm25-service"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["full"] }
tonic = "0.11"
prost = "0.12"
redis = { version = "0.24", features = ["tokio-comp", "connection-manager"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
metrics = "0.22"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tantivy = "0.22"
anyhow = "1.0"
thiserror = "1.0"

[build-dependencies]
tonic-build = "0.11"
```

#### 1.2 gRPC 协议定义

**任务描述**：定义 BM25 服务的 gRPC 接口

**具体步骤**：
1. 创建 `proto/bm25.proto` 文件
2. 定义服务接口和消息类型
3. 配置 `build.rs` 生成 Rust 代码

**产出物**：
- `services/bm25/proto/bm25.proto`
- `services/bm25/build.rs`

**代码示例**：

```protobuf
syntax = "proto3";

package bm25;

service BM25Service {
  rpc IndexDocument(IndexDocumentRequest) returns (IndexDocumentResponse);
  rpc BatchIndexDocuments(BatchIndexDocumentsRequest) returns (BatchIndexDocumentsResponse);
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc DeleteDocument(DeleteDocumentRequest) returns (DeleteDocumentResponse);
  rpc GetStats(GetStatsRequest) returns (GetStatsResponse);
}

message IndexDocumentRequest {
  string index_name = 1;
  string document_id = 2;
  map<string, string> fields = 3;
}

message IndexDocumentResponse {
  bool success = 1;
  string message = 2;
}

message BatchIndexDocumentsRequest {
  string index_name = 1;
  repeated Document documents = 2;
}

message Document {
  string document_id = 1;
  map<string, string> fields = 2;
}

message BatchIndexDocumentsResponse {
  bool success = 1;
  string message = 2;
  int32 indexed_count = 3;
}

message SearchRequest {
  string index_name = 1;
  string query = 2;
  int32 limit = 3;
  int32 offset = 4;
  map<string, float> field_weights = 5;
  bool highlight = 6;
}

message SearchResponse {
  repeated SearchResult results = 1;
  int32 total = 2;
  float max_score = 3;
}

message SearchResult {
  string document_id = 1;
  float score = 2;
  map<string, string> fields = 3;
  map<string, string> highlights = 4;
}

message DeleteDocumentRequest {
  string index_name = 1;
  string document_id = 2;
}

message DeleteDocumentResponse {
  bool success = 1;
  string message = 2;
}

message GetStatsRequest {
  string index_name = 1;
}

message GetStatsResponse {
  int64 total_documents = 1;
  int64 total_terms = 2;
  double avg_document_length = 3;
}
```

#### 1.3 配置管理

**任务描述**：实现配置加载和管理模块

**具体步骤**：
1. 创建配置结构体
2. 实现从 TOML 文件加载配置
3. 支持环境变量覆盖

**产出物**：
- `services/bm25/src/config/mod.rs`
- `services/bm25/src/config/config.rs`
- `services/bm25/configs/config.toml`

**代码示例**：

```rust
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub redis: RedisConfig,
    pub index: IndexConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub address: SocketAddr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexConfig {
    pub data_dir: String,
    pub index_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ttl_seconds: u64,
    pub max_size: usize,
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}
```

#### 1.4 日志和监控

**任务描述**：集成 Tracing 日志系统和 Metrics 监控

**具体步骤**：
1. 配置 Tracing subscriber
2. 初始化 Metrics exporter
3. 定义关键指标

**产出物**：
- `services/bm25/src/main.rs` (日志初始化)
- `services/bm25/src/metrics/mod.rs`

**代码示例**：

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bm25_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub fn init_metrics() {
    metrics::describe_counter!("bm25_search_requests_total", "Total number of search requests");
    metrics::describe_counter!("bm25_index_documents_total", "Total number of indexed documents");
    metrics::describe_histogram!("bm25_search_duration_seconds", "Search request duration");
}
```

#### 1.5 gRPC 服务框架

**任务描述**：实现基础的 gRPC 服务框架

**具体步骤**：
1. 实现 gRPC 服务 trait
2. 创建 tonic 服务
3. 实现健康检查接口

**产出物**：
- `services/bm25/src/proto/mod.rs`
- `services/bm25/src/proto/service.rs`
- `services/bm25/src/main.rs` (服务启动)

**代码示例**：

```rust
use tonic::{transport::Server, Request, Response, Status};

pub struct BM25Service {
    config: Config,
}

impl BM25Service {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[tonic::async_trait]
impl bm25::bm25_service_server::Bm25Service for BM25Service {
    async fn index_document(
        &self,
        request: Request<IndexDocumentRequest>,
    ) -> Result<Response<IndexDocumentResponse>, Status> {
        tracing::info!("Received index document request");
        Ok(Response::new(IndexDocumentResponse {
            success: true,
            message: "Not implemented yet".to_string(),
        }))
    }
}
```

### 验收标准

- [ ] 项目结构完整，编译通过
- [ ] gRPC 协议定义清晰，代码生成成功
- [ ] 配置文件可正确加载
- [ ] 日志和监控系统正常工作
- [ ] gRPC 服务可正常启动和响应

---

## 阶段二：索引管理（Week 2-3）

### 目标

实现基于 Tantivy 的索引管理功能，包括索引创建、文档添加、更新和删除。

### 任务清单

#### 2.1 Tantivy 集成

**任务描述**：集成 Tantivy 库，创建索引管理器

**具体步骤**：
1. 创建 Tantivy 索引 schema
2. 实现索引管理器
3. 实现索引创建和打开

**产出物**：
- `services/bm25/src/index/mod.rs`
- `services/bm25/src/index/manager.rs`
- `services/bm25/src/index/schema.rs`

**代码示例**：

```rust
use tantivy::{
    schema::*,
    Index, IndexWriter, ReloadPolicy,
};
use std::path::Path;
use anyhow::Result;

pub struct IndexManager {
    index: Index,
    schema: Schema,
}

impl IndexManager {
    pub fn create<P: AsRef<Path>>(path: P) -> Result<Self> {
        let schema = Self::build_schema();
        let index = Index::create_in_dir(path, schema.clone())?;
        Ok(Self { index, schema })
    }

    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let index = Index::open_in_dir(path)?;
        let schema = index.schema();
        Ok(Self { index, schema: schema.clone() })
    }

    fn build_schema() -> Schema {
        let mut schema_builder = Schema::builder();
        schema_builder.add_text_field("document_id", STRING | STORED);
        schema_builder.add_text_field("title", TEXT | STORED);
        schema_builder.add_text_field("content", TEXT | STORED);
        schema_builder.build()
    }

    pub fn writer(&self) -> Result<IndexWriter> {
        Ok(self.index.writer(50_000_000)?)
    }

    pub fn reader(&self) -> Result<IndexReader> {
        Ok(self.index.reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()?)
    }
}
```

#### 2.2 文档索引

**任务描述**：实现文档添加和更新功能

**具体步骤**：
1. 实现文档到 Tantivy Document 的转换
2. 实现文档添加逻辑
3. 实现文档更新逻辑

**产出物**：
- `services/bm25/src/index/document.rs`

**代码示例**：

```rust
use tantivy::Document;
use std::collections::HashMap;

pub fn to_document(document_id: &str, fields: &HashMap<String, String>) -> Document {
    let mut doc = Document::new();
    doc.add_text("document_id", document_id);
    
    for (key, value) in fields {
        if key == "title" {
            doc.add_text("title", value);
        } else if key == "content" {
            doc.add_text("content", value);
        }
    }
    
    doc
}

pub fn add_document(manager: &IndexManager, document_id: &str, fields: &HashMap<String, String>) -> Result<()> {
    let mut writer = manager.writer()?;
    let doc = to_document(document_id, fields);
    writer.add_document(doc)?;
    writer.commit()?;
    Ok(())
}

pub fn update_document(manager: &IndexManager, document_id: &str, fields: &HashMap<String, String>) -> Result<()> {
    let mut writer = manager.writer()?;
    let doc = to_document(document_id, fields);
    let term = Term::from_field_text(manager.schema.get_field("document_id")?, document_id);
    writer.update_document(term, doc)?;
    writer.commit()?;
    Ok(())
}
```

#### 2.3 文档删除

**任务描述**：实现文档删除功能

**具体步骤**：
1. 实现基于 document_id 的删除
2. 实现批量删除

**产出物**：
- `services/bm25/src/index/delete.rs`

**代码示例**：

```rust
use tantivy::Term;

pub fn delete_document(manager: &IndexManager, document_id: &str) -> Result<()> {
    let mut writer = manager.writer()?;
    let term = Term::from_field_text(manager.schema.get_field("document_id")?, document_id);
    writer.delete_term(term)?;
    writer.commit()?;
    Ok(())
}

pub fn batch_delete_documents(manager: &IndexManager, document_ids: &[String]) -> Result<usize> {
    let mut writer = manager.writer()?;
    let field = manager.schema.get_field("document_id")?;
    
    for doc_id in document_ids {
        let term = Term::from_field_text(field, doc_id);
        writer.delete_term(term);
    }
    
    writer.commit()?;
    Ok(document_ids.len())
}
```

#### 2.4 批量操作

**任务描述**：实现批量索引和批量删除

**具体步骤**：
1. 实现批量添加文档
2. 实现批量更新文档
3. 优化批量操作性能

**产出物**：
- `services/bm25/src/index/batch.rs`

**代码示例**：

```rust
pub fn batch_add_documents(manager: &IndexManager, documents: Vec<(String, HashMap<String, String>)>) -> Result<usize> {
    let mut writer = manager.writer()?;
    
    for (doc_id, fields) in documents {
        let doc = to_document(&doc_id, &fields);
        writer.add_document(doc)?;
    }
    
    writer.commit()?;
    Ok(documents.len())
}
```

#### 2.5 统计信息

**任务描述**：实现索引统计信息查询

**具体步骤**：
1. 实现文档总数统计
2. 实现词项总数统计
3. 实现平均文档长度计算

**产出物**：
- `services/bm25/src/index/stats.rs`

**代码示例**：

```rust
use tantivy::Searcher;

pub struct IndexStats {
    pub total_documents: u64,
    pub total_terms: u64,
    pub avg_document_length: f64,
}

pub fn get_stats(manager: &IndexManager) -> Result<IndexStats> {
    let reader = manager.reader()?;
    let searcher = reader.searcher();
    
    let total_documents = searcher.num_docs();
    let title_field = manager.schema.get_field("title")?;
    let content_field = manager.schema.get_field("content")?;
    
    let mut total_length = 0u64;
    for segment_reader in searcher.segment_readers() {
        total_length += segment_reader.num_tokens(title_field)?;
        total_length += segment_reader.num_tokens(content_field)?;
    }
    
    let avg_document_length = if total_documents > 0 {
        total_length as f64 / total_documents as f64
    } else {
        0.0
    };
    
    Ok(IndexStats {
        total_documents,
        total_terms: total_length,
        avg_document_length,
    })
}
```

### 验收标准

- [ ] 索引可正常创建和打开
- [ ] 文档可成功添加到索引
- [ ] 文档可成功更新
- [ ] 文档可成功删除
- [ ] 批量操作性能良好
- [ ] 统计信息准确

---

## 阶段三：搜索功能（Week 4-5）

### 目标

实现基于 BM25 算法的全文搜索功能，包括查询解析、结果排序和高亮显示。

### 任务清单

#### 3.1 查询解析

**任务描述**：实现查询语句解析和转换

**具体步骤**：
1. 实现基础查询解析
2. 支持布尔查询（AND、OR、NOT）
3. 支持短语查询
4. 支持字段查询

**产出物**：
- `services/bm25/src/search/parser.rs`

**代码示例**：

```rust
use tantivy::query::*;
use tantivy::schema::Field;

pub fn parse_query(index_manager: &IndexManager, query_str: &str) -> Result<Box<dyn Query>> {
    let title_field = index_manager.schema.get_field("title")?;
    let content_field = index_manager.schema.get_field("content")?;
    
    let query_parser = QueryParser::for_index(
        &index_manager.index,
        vec![title_field, content_field],
    );
    
    let query = query_parser.parse_query(query_str)?;
    Ok(query)
}

pub fn parse_field_query(index_manager: &IndexManager, field_name: &str, query_str: &str) -> Result<Box<dyn Query>> {
    let field = index_manager.schema.get_field(field_name)?;
    let query_parser = QueryParser::for_index(&index_manager.index, vec![field]);
    let query = query_parser.parse_query(query_str)?;
    Ok(query)
}
```

#### 3.2 BM25 搜索

**任务描述**：实现基于 BM25 的搜索功能

**具体步骤**：
1. 使用 Tantivy 的 BM25 评分
2. 实现搜索管理器
3. 支持字段权重配置

**产出物**：
- `services/bm25/src/search/manager.rs`
- `services/bm25/src/search/bm25.rs`

**代码示例**：

```rust
use tantivy::{collector::TopDocs, Score, Searcher};
use std::collections::HashMap;

pub struct SearchResult {
    pub document_id: String,
    pub score: Score,
    pub fields: HashMap<String, String>,
}

pub struct SearchManager {
    index_manager: IndexManager,
}

impl SearchManager {
    pub fn new(index_manager: IndexManager) -> Self {
        Self { index_manager }
    }
    
    pub fn search(&self, query: &dyn Query, limit: usize, offset: usize) -> Result<Vec<SearchResult>> {
        let reader = self.index_manager.reader()?;
        let searcher = reader.searcher();
        
        let top_docs = searcher.search(query, &TopDocs::with_limit(limit).and_offset(offset))?;
        
        let mut results = Vec::new();
        for (score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            let document_id = retrieved_doc
                .get_first(self.index_manager.schema.get_field("document_id")?)
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            
            let mut fields = HashMap::new();
            if let Some(title) = retrieved_doc.get_first(self.index_manager.schema.get_field("title")?) {
                fields.insert("title".to_string(), title.as_str().unwrap().to_string());
            }
            if let Some(content) = retrieved_doc.get_first(self.index_manager.schema.get_field("content")?) {
                fields.insert("content".to_string(), content.as_str().unwrap().to_string());
            }
            
            results.push(SearchResult {
                document_id,
                score,
                fields,
            });
        }
        
        Ok(results)
    }
}
```

#### 3.3 字段加权

**任务描述**：实现字段级别的权重配置

**具体步骤**：
1. 实现字段权重映射
2. 在搜索时应用字段权重
3. 支持动态权重调整

**产出物**：
- `services/bm25/src/search/weight.rs`

**代码示例**：

```rust
use tantivy::query::BoostQuery;
use std::collections::HashMap;

pub fn apply_field_weights(query: Box<dyn Query>, field_weights: &HashMap<String, f32>) -> Box<dyn Query> {
    if field_weights.is_empty() {
        return query;
    }
    
    let mut boosted_queries: Vec<Box<dyn Query>> = Vec::new();
    
    for (field_name, weight) in field_weights {
        let field_query = parse_field_query(&index_manager, field_name, &query_str)?;
        let boosted_query = BoostQuery::new(field_query, *weight);
        boosted_queries.push(Box::new(boosted_query));
    }
    
    if boosted_queries.len() == 1 {
        boosted_queries.pop().unwrap()
    } else {
        Box::new(BooleanQuery::union(boosted_queries))
    }
}
```

#### 3.4 高亮显示

**任务描述**：实现搜索结果高亮显示

**具体步骤**：
1. 使用 Tantivy 的高亮功能
2. 配置高亮样式
3. 提取上下文片段

**产出物**：
- `services/bm25/src/search/highlight.rs`

**代码示例**：

```rust
use tantivy::highlight::{Highlight, HighlightGenerator};
use std::collections::HashMap;

pub fn highlight_results(
    searcher: &Searcher,
    query: &dyn Query,
    results: &[SearchResult],
) -> Result<HashMap<String, HashMap<String, String>>> {
    let highlighter = Highlight::with_tag("<em>", "</em>");
    let mut highlights = HashMap::new();
    
    let title_field = index_manager.schema.get_field("title")?;
    let content_field = index_manager.schema.get_field("content")?;
    
    for result in results {
        let doc_address = searcher.doc_by_id(result.document_id.parse::<u64>()?)?;
        let mut field_highlights = HashMap::new();
        
        let mut highlight_generator = highlighter.highlight("title", query, searcher, doc_address)?;
        if let Some(highlighted) = highlight_generator.next() {
            field_highlights.insert("title".to_string(), highlighted?);
        }
        
        let mut highlight_generator = highlighter.highlight("content", query, searcher, doc_address)?;
        if let Some(highlighted) = highlight_generator.next() {
            field_highlights.insert("content".to_string(), highlighted?);
        }
        
        highlights.insert(result.document_id.clone(), field_highlights);
    }
    
    Ok(highlights)
}
```

#### 3.5 gRPC 搜索接口

**任务描述**：实现 gRPC 搜索接口

**具体步骤**：
1. 实现 Search RPC 方法
2. 处理搜索参数
3. 返回格式化结果

**产出物**：
- `services/bm25/src/proto/service.rs` (更新)

**代码示例**：

```rust
async fn search(
    &self,
    request: Request<SearchRequest>,
) -> Result<Response<SearchResponse>, Status> {
    let req = request.into_inner();
    
    metrics::counter!("bm25_search_requests_total").increment(1);
    let start = std::time::Instant::now();
    
    let index_manager = self.index_manager.lock().await;
    let search_manager = SearchManager::new(index_manager.clone());
    
    let query = parse_query(&index_manager, &req.query)
        .map_err(|e| Status::internal(format!("Failed to parse query: {}", e)))?;
    
    let results = search_manager.search(&query, req.limit as usize, req.offset as usize)
        .map_err(|e| Status::internal(format!("Search failed: {}", e)))?;
    
    let mut proto_results = Vec::new();
    for result in results {
        proto_results.push(SearchResult {
            document_id: result.document_id,
            score: result.score,
            fields: result.fields,
            highlights: HashMap::new(),
        });
    }
    
    let duration = start.elapsed();
    metrics::histogram!("bm25_search_duration_seconds").record(duration.as_secs_f64());
    
    Ok(Response::new(SearchResponse {
        results: proto_results,
        total: proto_results.len() as i32,
        max_score: proto_results.first().map(|r| r.score).unwrap_or(0.0),
    }))
}
```

### 验收标准

- [ ] 查询可正确解析
- [ ] BM25 评分正常工作
- [ ] 字段权重生效
- [ ] 高亮显示正确
- [ ] gRPC 接口正常响应
- [ ] 搜索性能满足要求

---

## 阶段四：缓存和持久化（Week 6）

### 目标

实现查询缓存和索引持久化功能，提升性能和数据可靠性。

### 任务清单

#### 4.1 Redis 缓存

**任务描述**：实现基于 Redis 的查询缓存

**具体步骤**：
1. 创建 Redis 连接池
2. 实现缓存键生成
3. 实现缓存读写
4. 实现 TTL 过期

**产出物**：
- `services/bm25/src/cache/mod.rs`
- `services/bm25/src/cache/redis.rs`

**代码示例**：

```rust
use redis::{AsyncCommands, Client, ConnectionManager};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct RedisCache {
    manager: ConnectionManager,
    ttl: Duration,
}

impl RedisCache {
    pub async fn new(url: &str, ttl: Duration) -> Result<Self> {
        let client = Client::open(url)?;
        let manager = client.get_connection_manager().await?;
        Ok(Self { manager, ttl })
    }
    
    pub fn cache_key(index_name: &str, query: &str, limit: u32, offset: u32) -> String {
        format!("bm25:{}:{}:{}:{}", index_name, query, limit, offset)
    }
    
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>> {
        let value: Option<String> = self.manager.get(key).await?;
        match value {
            Some(v) => {
                let result: T = serde_json::from_str(&v)?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }
    
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let serialized = serde_json::to_string(value)?;
        self.manager.set_ex(key, serialized, self.ttl.as_secs()).await?;
        Ok(())
    }
}
```

#### 4.2 缓存集成

**任务描述**：将缓存集成到搜索流程

**具体步骤**：
1. 在搜索前检查缓存
2. 缓存未命中时执行搜索
3. 搜索结果写入缓存
4. 统计缓存命中率

**产出物**：
- `services/bm25/src/search/manager.rs` (更新)

**代码示例**：

```rust
impl SearchManager {
    pub async fn search_with_cache(
        &self,
        cache: &RedisCache,
        query: &dyn Query,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<SearchResult>> {
        let cache_key = RedisCache::cache_key(&self.index_name, &query_str, limit as u32, offset as u32);
        
        if let Some(cached) = cache.get::<Vec<SearchResult>>(&cache_key).await? {
            metrics::counter!("bm25_cache_hits").increment(1);
            return Ok(cached);
        }
        
        metrics::counter!("bm25_cache_misses").increment(1);
        let results = self.search(query, limit, offset)?;
        cache.set(&cache_key, &results).await?;
        
        Ok(results)
    }
}
```

#### 4.3 索引持久化

**任务描述**：实现索引到 Redis 的持久化

**具体步骤**：
1. 实现索引序列化
2. 将索引元数据存储到 Redis
3. 支持从 Redis 恢复索引

**产出物**：
- `services/bm25/src/storage/mod.rs`
- `services/bm25/src/storage/redis.rs`

**代码示例**：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexMetadata {
    pub index_name: String,
    pub document_count: u64,
    pub last_updated: i64,
}

pub struct RedisStorage {
    manager: ConnectionManager,
}

impl RedisStorage {
    pub async fn save_metadata(&self, metadata: &IndexMetadata) -> Result<()> {
        let key = format!("bm25:index:metadata:{}", metadata.index_name);
        let value = serde_json::to_string(metadata)?;
        self.manager.set(&key, value).await?;
        Ok(())
    }
    
    pub async fn load_metadata(&self, index_name: &str) -> Result<Option<IndexMetadata>> {
        let key = format!("bm25:index:metadata:{}", index_name);
        let value: Option<String> = self.manager.get(&key).await?;
        match value {
            Some(v) => {
                let metadata: IndexMetadata = serde_json::from_str(&v)?;
                Ok(Some(metadata))
            }
            None => Ok(None),
        }
    }
}
```

#### 4.4 批量操作优化

**任务描述**：优化批量索引和删除操作

**具体步骤**：
1. 实现批量提交
2. 优化内存使用
3. 添加进度反馈

**产出物**：
- `services/bm25/src/index/batch.rs` (更新)

**代码示例**：

```rust
pub fn batch_add_documents_optimized(
    manager: &IndexManager,
    documents: Vec<(String, HashMap<String, String>)>,
    batch_size: usize,
) -> Result<usize> {
    let mut writer = manager.writer()?;
    let mut indexed_count = 0;
    
    for batch in documents.chunks(batch_size) {
        for (doc_id, fields) in batch {
            let doc = to_document(doc_id, fields);
            writer.add_document(doc)?;
            indexed_count += 1;
        }
        writer.commit()?;
    }
    
    Ok(indexed_count)
}
```

### 验收标准

- [ ] Redis 缓存正常工作
- [ ] 缓存命中率统计准确
- [ ] 索引元数据可持久化
- [ ] 批量操作性能优化
- [ ] 缓存过期机制正常

---

## 阶段五：测试和优化（Week 7-8）

### 目标

完善测试覆盖，优化性能，准备生产部署。

### 任务清单

#### 5.1 单元测试

**任务描述**：编写完整的单元测试

**具体步骤**：
1. 测试索引管理
2. 测试搜索功能
3. 测试缓存功能
4. 测试持久化功能

**产出物**：
- `services/bm25/src/index/tests.rs`
- `services/bm25/src/search/tests.rs`
- `services/bm25/src/cache/tests.rs`

**代码示例**：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_index_document() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let manager = IndexManager::create(&index_path).unwrap();
        
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Document".to_string());
        fields.insert("content".to_string(), "This is a test document content".to_string());
        
        add_document(&manager, "doc1", &fields).unwrap();
        
        let stats = get_stats(&manager).unwrap();
        assert_eq!(stats.total_documents, 1);
    }
    
    #[tokio::test]
    async fn test_search() {
        let temp_dir = TempDir::new().unwrap();
        let index_path = temp_dir.path().join("test_index");
        let manager = IndexManager::create(&index_path).unwrap();
        
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), "Test Document".to_string());
        fields.insert("content".to_string(), "This is a test document content".to_string());
        
        add_document(&manager, "doc1", &fields).unwrap();
        
        let search_manager = SearchManager::new(manager);
        let query = parse_query(&search_manager.index_manager, "test").unwrap();
        let results = search_manager.search(&query, 10, 0).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document_id, "doc1");
    }
}
```

#### 5.2 集成测试

**任务描述**：编写端到端集成测试

**具体步骤**：
1. 测试 gRPC 接口
2. 测试完整工作流程
3. 测试错误处理

**产出物**：
- `services/bm25/tests/integration_test.rs`

**代码示例**：

```rust
use bm25::proto::bm25_service_client::Bm25ServiceClient;
use tonic::transport::Channel;

#[tokio::test]
async fn test_index_and_search() {
    let channel = Channel::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();
    let mut client = Bm25ServiceClient::new(channel);
    
    let mut fields = HashMap::new();
    fields.insert("title".to_string(), "Integration Test".to_string());
    fields.insert("content".to_string(), "This is an integration test".to_string());
    
    let request = IndexDocumentRequest {
        index_name: "test".to_string(),
        document_id: "integration_doc1".to_string(),
        fields,
    };
    
    let response = client.index_document(request).await.unwrap().into_inner();
    assert!(response.success);
    
    let search_request = SearchRequest {
        index_name: "test".to_string(),
        query: "integration".to_string(),
        limit: 10,
        offset: 0,
        field_weights: HashMap::new(),
        highlight: true,
    };
    
    let search_response = client.search(search_request).await.unwrap().into_inner();
    assert_eq!(search_response.results.len(), 1);
}
```

#### 5.3 性能测试

**任务描述**：进行性能基准测试

**具体步骤**：
1. 测试索引性能
2. 测试搜索性能
3. 测试缓存效果
4. 识别性能瓶颈

**产出物**：
- `services/bm25/benches/benchmark.rs`

**代码示例**：

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_index_document(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("bench_index");
    let manager = IndexManager::create(&index_path).unwrap();
    
    c.bench_function("index_document", |b| {
        b.iter(|| {
            let mut fields = HashMap::new();
            fields.insert("title".to_string(), "Benchmark Document".to_string());
            fields.insert("content".to_string(), "This is a benchmark document content".to_string());
            add_document(black_box(&manager), "doc1", black_box(&fields)).unwrap();
        });
    });
}

fn bench_search(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path().join("bench_index");
    let manager = IndexManager::create(&index_path).unwrap();
    
    for i in 0..1000 {
        let mut fields = HashMap::new();
        fields.insert("title".to_string(), format!("Document {}", i));
        fields.insert("content".to_string(), format!("Content for document {}", i));
        add_document(&manager, &format!("doc{}", i), &fields).unwrap();
    }
    
    let search_manager = SearchManager::new(manager);
    let query = parse_query(&search_manager.index_manager, "document").unwrap();
    
    c.bench_function("search", |b| {
        b.iter(|| {
            search_manager.search(black_box(&query), 10, 0).unwrap();
        });
    });
}

criterion_group!(benches, bench_index_document, bench_search);
criterion_main!(benches);
```

#### 5.4 性能优化

**任务描述**：基于测试结果进行性能优化

**具体步骤**：
1. 优化索引构建速度
2. 优化搜索查询速度
3. 优化内存使用
4. 优化并发性能

**优化方向**：
- 使用批量提交减少 I/O
- 优化查询缓存策略
- 使用连接池管理 Redis 连接
- 实现查询结果预加载

#### 5.5 错误处理完善

**任务描述**：完善错误处理和日志

**具体步骤**：
1. 定义自定义错误类型
2. 实现错误转换
3. 添加详细的错误日志
4. 实现错误恢复机制

**产出物**：
- `services/bm25/src/error.rs`

**代码示例**：

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Bm25Error {
    #[error("Index not found: {0}")]
    IndexNotFound(String),
    
    #[error("Document not found: {0}")]
    DocumentNotFound(String),
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Cache error: {0}")]
    CacheError(#[from] redis::RedisError),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Tantivy error: {0}")]
    TantivyError(#[from] tantivy::TantivyError),
}
```

#### 5.6 文档和部署

**任务描述**：完善文档和部署配置

**具体步骤**：
1. 编写 API 文档
2. 编写部署文档
3. 创建 Dockerfile
4. 编写 Kubernetes 配置

**产出物**：
- `services/bm25/README.md`
- `services/bm25/Dockerfile`
- `services/bm25/k8s/deployment.yaml`

**Dockerfile 示例**：

```dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/bm25-service /app/bm25-service
COPY --from=builder /app/configs /app/configs

EXPOSE 50051

CMD ["./bm25-service"]
```

### 验收标准

- [ ] 单元测试覆盖率 > 80%
- [ ] 集成测试全部通过
- [ ] 性能指标满足要求
- [ ] 错误处理完善
- [ ] 文档完整清晰
- [ ] 可成功部署

---

## 总结

### 交付物清单

| 阶段 | 交付物 |
|------|--------|
| 阶段一 | 项目结构、gRPC 协议、配置管理、日志监控 |
| 阶段二 | 索引管理、文档 CRUD、批量操作、统计信息 |
| 阶段三 | 搜索功能、BM25 评分、字段加权、高亮显示 |
| 阶段四 | Redis 缓存、索引持久化、批量优化 |
| 阶段五 | 单元测试、集成测试、性能测试、文档部署 |

### 关键指标

| 指标 | 目标值 |
|------|--------|
| 索引吞吐量 | > 1000 docs/s |
| 搜索延迟 | P99 < 100ms |
| 缓存命中率 | > 70% |
| 测试覆盖率 | > 80% |
| 内存占用 | < 2GB (100万文档) |

### 后续优化方向

1. **分布式索引**：支持索引分片和分布式搜索
2. **实时更新**：支持近实时的索引更新
3. **高级查询**：支持范围查询、模糊查询、同义词扩展
4. **性能优化**：进一步优化搜索和索引性能
5. **监控告警**：完善监控指标和告警机制

### 风险和挑战

1. **Tantivy 学习曲线**：需要熟悉 Tantivy 的 API 和最佳实践
2. **性能调优**：BM25 参数调优需要大量测试
3. **并发控制**：索引和搜索的并发访问需要仔细设计
4. **内存管理**：大规模索引的内存占用需要优化
5. **数据一致性**：缓存和持久化的数据一致性需要保证

---

## 参考资料

- [Tantivy 官方文档](https://docs.rs/tantivy/)
- [BM25 算法详解](https://en.wikipedia.org/wiki/Okapi_BM25)
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [gRPC Rust 教程](https://github.com/hyperium/tonic)
- [Redis Rust 客户端](https://docs.rs/redis/)
