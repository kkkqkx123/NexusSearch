# ref/bm25 服务端功能补充设计方案

## 一、背景说明

本文档基于 `docs/bm25/关闭方法设计方案.md` 的分析结论，明确 ref/bm25 服务端需要补充的功能。

**分析结论回顾：**
1. ✅ **客户端** - 需要实现 `disconnect()` 方法（已完成）
2. ❌ **服务端 Shutdown** - 不需要添加 shutdown 端点
3. ✅ **服务端 ClearIndex** - 需要添加 clear_index 端点

## 二、功能需求分析

### 2.1 不需要的功能：Shutdown 端点

**结论：❌ 不需要实现**

#### 理由：

1. **无状态设计**
   - ref/bm25 是无状态服务，不维护客户端连接状态
   - 服务端不知道有哪些客户端连接
   - 无法实现"通知所有客户端关闭"的功能

2. **gRPC 框架支持**
   - tonic::Server 已提供 `serve_with_graceful_shutdown` 机制
   - 可以优雅地停止接收新请求
   - 等待现有请求完成

3. **服务独立性**
   - 服务端可以独立重启，不影响客户端
   - 客户端会自动重连或报错
   - 符合微服务设计原则

4. **已有实现**
   ```rust
   // ref/bm25/src/main.rs
   let server = Server::builder()
       .add_service(Bm25ServiceServer::new(bm25_service))
       .serve_with_graceful_shutdown(addr, shutdown_signal)
       .await?;
   ```

#### 如果需要优雅关闭：

**正确做法：**
```rust
use tokio::signal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

    // 监听 Ctrl+C 信号
    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        tracing::info!("Received shutdown signal");
        shutdown_tx.send(()).ok();
    });

    let server = Server::builder()
        .add_service(Bm25ServiceServer::new(bm25_service))
        .serve_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
            tracing::info!("Graceful shutdown initiated");
        })
        .await?;

    Ok(())
}
```

### 2.2 需要的功能：ClearIndex 端点

**结论：✅ 需要实现**

#### 理由：

1. **客户端需求**
   - `src/api/handlers/index_management.rs` 中的 `handle_clear_index` 需要清除 BM25 索引
   - 当前只能报告文档数，无法实际清除
   - 影响索引管理功能的完整性

2. **运维需求**
   - 需要定期清理旧索引
   - 需要重建索引（先清空再重建）
   - 需要测试环境中的快速重置

3. **对标其他存储**
   - Qdrant 有 `clear_collection()` 方法
   - SQLite 有 `DELETE` 语句
   - BM25 也应该提供类似功能

#### 当前限制：

```rust
// src/api/handlers/index_management.rs
if request.clear_bm25 {
    if let Some(ref bm25) = state.bm25 {
        let mut client = bm25.lock().await;
        // Note: BM25 service doesn't have a clear_index endpoint
        // For now, we can only report the current document count
        if let Ok(stats) = client.stats("default").await {
            bm25_cleared = stats.total_documents as usize;
            tracing::warn!(
                "BM25 clearing requested but not implemented. Current documents: {}. \
                 Service endpoint 'clear_index' needs to be added to ref/bm25.",
                bm25_cleared
            );
        }
    }
}
```

## 三、ClearIndex 端点设计方案

### 3.1 Proto 定义

在 `ref/bm25/proto/bm25.proto` 中添加：

```protobuf
syntax = "proto3";

package bm25;

service BM25Service {
  rpc IndexDocument(IndexDocumentRequest) returns (IndexDocumentResponse);
  rpc BatchIndexDocuments(BatchIndexDocumentsRequest) returns (BatchIndexDocumentsResponse);
  rpc Search(SearchRequest) returns (SearchResponse);
  rpc DeleteDocument(DeleteDocumentRequest) returns (DeleteDocumentResponse);
  rpc GetStats(GetStatsRequest) returns (GetStatsResponse);

  // 新增：清除索引
  rpc ClearIndex(ClearIndexRequest) returns (ClearIndexResponse);
}

// ... 现有消息定义 ...

message ClearIndexRequest {
  string index_name = 1;
}

message ClearIndexResponse {
  bool success = 1;
  string message = 2;
  int32 cleared_count = 3;  // 清除的文档数量
}
```

### 3.2 服务端实现

在 `ref/bm25/src/main.rs` 中添加实现：

```rust
#[tonic::async_trait]
impl Bm25ServiceTrait for BM25Service {
    // ... 现有方法 ...

    async fn clear_index(
        &self,
        request: Request<ClearIndexRequest>,
    ) -> Result<Response<ClearIndexResponse>, Status> {
        let req = request.into_inner();
        tracing::info!("Received clear index request: index={}", req.index_name);

        // 获取清除前的文档数量
        let (manager, _schema) = self.get_or_create_index(&req.index_name).await;
        let stats = stats::get_stats(&manager).map_err(|e| {
            Status::internal(format!("Failed to get stats: {}", e))
        })?;

        let cleared_count = stats.total_documents as i32;

        // 删除索引目录
        let index_path = self.index_path.join(&req.index_name);
        if index_path.exists() {
            std::fs::remove_dir_all(&index_path).map_err(|e| {
                Status::internal(format!("Failed to remove index directory: {}", e))
            })?;

            // 从内存中移除索引引用
            let mut indexes = self.indexes.write().await;
            indexes.remove(&req.index_name);

            tracing::info!("Cleared index '{}': {} documents removed", req.index_name, cleared_count);
        }

        Ok(Response::new(ClearIndexResponse {
            success: true,
            message: format!("Index '{}' cleared successfully", req.index_name),
            cleared_count,
        }))
    }
}
```

### 3.3 客户端实现

在 `src/storage/bm25/client.rs` 中添加方法：

```rust
impl Bm25Client {
    /// Clear all documents in an index
    ///
    /// This will remove all documents from the specified index.
    /// Note: This operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `index_name` - The name of the index to clear
    ///
    /// # Returns
    ///
    /// Returns the number of documents that were cleared
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// client.clear_index("code_index").await?;
    /// ```
    pub async fn clear_index(&mut self, index_name: &str) -> Result<usize, Bm25Error> {
        let client = self.client.as_mut().ok_or(Bm25Error::Disabled)?;

        let request = tonic::Request::new(pb::ClearIndexRequest {
            index_name: index_name.to_string(),
        });

        let response = client
            .clear_index(request)
            .await
            .map_err(Bm25Error::Grpc)?;

        let result = response.into_inner();

        if result.success {
            tracing::info!(
                "Cleared index '{}': {} documents removed",
                index_name,
                result.cleared_count
            );
            Ok(result.cleared_count as usize)
        } else {
            Err(Bm25Error::index(format!(
                "Failed to clear index '{}': {}",
                index_name, result.message
            )))
        }
    }
}
```

### 3.4 集成到 API Handler

修改 `src/api/handlers/index_management.rs`：

```rust
// Clear BM25 if requested
if request.clear_bm25 {
    if let Some(ref bm25) = state.bm25 {
        let mut client = bm25.lock().await;
        match client.clear_index("default").await {
            Ok(count) => {
                bm25_cleared = count;
                tracing::info!("Cleared {} documents from BM25", count);
            }
            Err(e) => {
                tracing::error!("Failed to clear BM25: {}", e);
                // 继续执行，不中断整个清除流程
            }
        }
    }
}
```

## 四、实现步骤

### 步骤 1：修改 Proto 定义

**文件：** `ref/bm25/proto/bm25.proto`

1. 添加 `ClearIndexRequest` 和 `ClearIndexResponse` 消息
2. 在 `BM25Service` 中添加 `rpc ClearIndex` 方法

### 步骤 2：重新生成代码

在 `ref/bm25` 目录下运行：

```bash
cd ref/bm25
cargo build
```

这将自动生成新的 gRPC 代码。

### 步骤 3：实现服务端逻辑

**文件：** `ref/bm25/src/main.rs`

1. 在 `BM25Service` impl 中添加 `clear_index` 方法
2. 实现删除索引目录的逻辑
3. 更新内存中的索引引用

### 步骤 4：实现客户端方法

**文件：** `src/storage/bm25/client.rs`

1. 添加 `clear_index()` 方法
2. 处理响应和错误

### 步骤 5：更新 API Handler

**文件：** `src/api/handlers/index_management.rs`

1. 替换当前的警告日志为实际的清除调用
2. 处理清除成功和失败的情况

### 步骤 6：添加测试

**服务端测试：**
```rust
#[tokio::test]
async fn test_clear_index() {
    let config = Config::default();
    let service = BM25Service::new(config);

    // 先添加一些文档
    // ... 添加文档代码 ...

    // 清除索引
    let request = Request::new(ClearIndexRequest {
        index_name: "test_index".to_string(),
    });
    let response = service.clear_index(request).await.unwrap();
    assert!(response.into_inner().success);

    // 验证索引已清空
    // ... 验证代码 ...
}
```

**客户端测试：**
```rust
#[tokio::test]
async fn test_clear_index() {
    let mut client = Bm25Client::new(test_config());

    // 添加一些文档
    // ... 添加文档代码 ...

    // 清除索引
    let count = client.clear_index("test_index").await.unwrap();
    assert!(count > 0);

    // 验证索引已清空
    let stats = client.stats("test_index").await.unwrap();
    assert_eq!(stats.total_documents, 0);
}
```

## 五、注意事项

### 5.1 安全性

1. **权限控制**（未来扩展）
   ```protobuf
   message ClearIndexRequest {
     string index_name = 1;
     string auth_token = 2;  // 可选的认证令牌
   }
   ```

2. **确认机制**（可选）
   - 对于生产环境，可以考虑添加确认参数
   - 防止误操作删除重要索引

### 5.2 性能考虑

1. **大索引处理**
   - 删除大索引可能需要较长时间
   - 考虑添加异步清除选项

2. **并发控制**
   - 清除操作应该独占索引
   - 防止在清除期间进行索引操作

### 5.3 错误处理

1. **索引不存在**
   ```rust
   if !index_path.exists() {
       return Ok(Response::new(ClearIndexResponse {
           success: true,
           message: format!("Index '{}' does not exist", req.index_name),
           cleared_count: 0,
       }));
   }
   ```

2. **权限不足**
   ```rust
   if !index_path.exists() {
       return Err(Status::permission_denied(
           format!("No permission to access index '{}'", req.index_name)
       ));
   }
   ```

## 六、替代方案（不推荐）

### 方案 A：逐个删除文档

```rust
// 不推荐：效率低
async fn clear_index_by_deletion(&self, index_name: &str) -> Result<usize> {
    // 1. 搜索所有文档
    let all_docs = self.search(index_name, "", 1000000).await?;

    // 2. 逐个删除
    for doc in &all_docs {
        self.delete(index_name, &doc.document_id).await?;
    }

    Ok(all_docs.len())
}
```

**缺点：**
- 效率极低
- 需要多次网络请求
- 无法保证原子性

### 方案 B：客户端删除索引目录

```rust
// 不推荐：破坏封装性
async fn clear_index_by_deleting_dir(&self, index_name: &str) -> Result<()> {
    let index_path = PathBuf::from("/data/bm25").join(index_name);
    std::fs::remove_dir_all(&index_path)?;
    Ok(())
}
```

**缺点：**
- 客户端需要知道服务端的文件系统结构
- 破坏了服务-客户端的封装
- 权限问题
- 跨机器部署时不可行

## 七、总结

### 7.1 功能需求总结

| 功能 | 是否需要 | 优先级 | 理由 |
|------|---------|--------|------|
| **Shutdown 端点** | ❌ 不需要 | - | gRPC 框架已支持优雅关闭 |
| **ClearIndex 端点** | ✅ 需要 | 高 | 客户端和管理功能需要 |

### 7.2 实现优先级

| 阶段 | 任务 | 工作量 | 优先级 |
|------|------|--------|--------|
| P0 | 修改 Proto 定义 | 0.5 小时 | 高 |
| P0 | 实现服务端逻辑 | 2 小时 | 高 |
| P0 | 实现客户端方法 | 1 小时 | 高 |
| P0 | 更新 API Handler | 0.5 小时 | 高 |
| P1 | 添加单元测试 | 2 小时 | 中 |
| P2 | 添加权限控制 | 4 小时 | 低 |
| P2 | 添加异步清除 | 4 小时 | 低 |

### 7.3 预期效果

**实现后：**
1. ✅ 可以通过 API 清除 BM25 索引
2. ✅ 支持索引重建和重置
3. ✅ 完善索引管理功能
4. ✅ 与 Qdrant 功能对齐

**不影响：**
1. ✅ 服务端仍然通过 tonic::Server 处理优雅关闭
2. ✅ 客户端可以独立断开连接
3. ✅ 服务可以独立重启

## 八、参考资料

- [Tantivy Documentation](https://docs.rs/tantivy/latest/tantivy/)
- [tonic gRPC Documentation](https://docs.rs/tonic/latest/tonic/)
- [gRPC Best Practices](https://grpc.io/docs/guides/)

---

**文档版本：** 1.0
**创建日期：** 2026-03-24
**作者：** Code Context Engine Team
**状态：** 待审核
**关联文档：** `docs/bm25/关闭方法设计方案.md`