# 向量搜索集成设计

## 一、模块概述

向量搜索功能通过直接集成 Qdrant 向量数据库实现，将向量搜索能力集成到查询协调器中，避免重复造轮子，降低开发和维护成本。Qdrant 是一个高性能的向量搜索引擎，专为向量相似度搜索优化，支持 HNSW 索引、混合搜索、过滤等高级功能。

---

## 二、核心功能

### 2.1 功能列表

| 功能 | 描述 | 实现方式 |
|------|------|---------|
| **文档索引** | 添加、更新、删除文档向量 | Qdrant API |
| **向量搜索** | 基于相似度的向量搜索 | Qdrant Search API |
| **批量操作** | 支持批量添加和删除 | Qdrant Batch API |
| **缓存管理** | 自动缓存热门查询 | Redis 缓存 |
| **向量归一化** | 支持向量归一化 | Qdrant 内置 |
| **监控指标** | 收集性能指标 | Prometheus + Qdrant Metrics |
| **混合搜索** | 结合关键词和向量搜索 | Qdrant Fusion API |

### 2.2 搜索特性

| 特性 | 支持 | 说明 |
|------|------|------|
| **余弦相似度** | ✅ | Qdrant 原生支持 |
| **欧氏距离** | ✅ | Qdrant 原生支持 |
| **点积** | ✅ | Qdrant 原生支持 |
| **HNSW 索引** | ✅ | Qdrant 内置优化 |
| **精确搜索** | ✅ | Qdrant 支持 |
| **混合搜索** | ✅ | 密集向量 + 稀疏向量融合 |
| **过滤查询** | ✅ | 基于 Payload 的过滤 |
| **向量量化** | ✅ | 减少 97% 内存占用 |
| **SIMD 加速** | ✅ | 自动利用硬件加速 |

---

## 三、架构设计

### 3.1 整体架构

```
┌─────────────────────────────────────────┐
│         API 网关                │
└──────────────┬──────────────────────┘
               │ gRPC
               ▼
┌─────────────────────────────────────────┐
│         查询协调器                │
│  ┌──────────────┬──────────────┐       │
│  │ 查询路由器   │ 结果融合器   │       │
│  └──────────────┴──────────────┘       │
│  ┌──────────────┬──────────────┐       │
│  │ FlexSearch   │ BM25         │       │
│  │ 客户端       │ 客户端       │       │
│  └──────────────┴──────────────┘       │
│  ┌──────────────┬──────────────┐       │
│  │ Qdrant       │ Redis 缓存   │       │
│  │ 客户端       │ 管理器       │       │
│  └──────────────┴──────────────┘       │
└──────────────┬──────────────────────┘
               │
        ┌──────┼──────┐
        │      │        │
        ▼      ▼        ▼
┌──────────┐ ┌──────────┐ ┌──────────┐
│Inversearch│ │  BM25    │ │  Qdrant  │
│ 服务      │ │  服务    │ │  服务    │
└──────────┘ └──────┬───┘ └──────┬───┘
                   │             │
                   ▼             ▼
              ┌──────────┐ ┌──────────┐
              │PostgreSQL│ │  Redis   │
              │          │ │  缓存    │
              └──────────┘ └──────────┘
```

### 3.2 目录结构

```
services/coordinator/
├── cmd/
│   └── main.go              # 入口文件
├── internal/
│   ├── config/              # 配置管理
│   │   ├── config.go       # 配置结构
│   │   └── engines.go      # 引擎配置
│   ├── router/              # 查询路由器
│   │   ├── router.go       # 路由决策
│   │   └── strategy.go     # 路由策略
│   ├── merger/              # 结果融合器
│   │   ├── merger.go       # 融合接口
│   │   ├── rrf.go          # RRF 策略
│   │   └── weighted.go     # 加权策略
│   ├── engine/              # 搜索引擎客户端
│   │   ├── client.go       # 客户端接口
│   │   ├── flexsearch.go   # FlexSearch 客户端
│   │   ├── bm25.go         # BM25 客户端
│   │   └── vector.go       # Qdrant 客户端
│   ├── cache/               # 缓存管理
│   │   └── redis.go        # Redis 缓存
│   ├── service/             # gRPC 服务
│   │   └── search.go       # 搜索服务实现
│   └── util/                # 工具函数
│       ├── error.go        # 错误处理
│       ├── logger.go       # 日志工具
│       └── metrics.go      # 监控指标
├── proto/                   # gRPC 协议定义
│   └── coordinator.proto
├── configs/                 # 配置文件
│   └── config.yaml
├── docker-compose.yml        # Docker Compose 配置
├── Dockerfile
└── go.mod                   # Go 模块配置
```

---

## 四、模块职责

### 4.1 Qdrant 客户端

**职责**：
- 封装 Qdrant API 调用
- 管理与 Qdrant 的连接
- 处理向量索引的创建和更新
- 处理向量搜索请求
- 处理批量操作
- 管理集合（Collection）生命周期

**主要逻辑**：
1. 初始化 Qdrant 客户端连接
2. 检查或创建集合
3. 添加/更新文档向量
4. 执行向量搜索
5. 处理过滤条件
6. 处理错误和重试

**数据结构**：
- Collection：向量集合，包含配置和索引
- Point：向量数据点，包含 ID、向量和 Payload
- Payload：与向量关联的元数据
- Filter：基于 Payload 的过滤条件

### 4.2 缓存管理器

**职责**：
- 缓存热门查询的结果
- 管理缓存的过期和淘汰
- 提供缓存命中率统计
- 支持缓存预热

**主要逻辑**：
1. 生成缓存键（基于查询向量和参数）
2. 检查缓存是否存在
3. 如果缓存命中，返回缓存结果
4. 如果缓存未命中，执行查询并缓存结果
5. 管理缓存的过期和淘汰
6. 统计缓存命中率

**缓存策略**：
- LRU：最近最少使用淘汰
- TTL：基于时间的过期
- 容量限制：限制缓存大小

### 4.3 查询路由器

**职责**：
- 根据查询特征决定使用哪些搜索引擎
- 支持动态路由策略
- 支持权重配置
- 支持 A/B 测试

**主要逻辑**：
1. 分析查询特征（长度、类型、特殊字符等）
2. 根据配置决定使用哪些引擎
3. 支持关键词搜索、向量搜索、混合搜索
4. 支持基于用户或场景的路由策略

**路由策略**：
- 纯关键词：仅使用 FlexSearch/BM25
- 纯语义：仅使用 Qdrant 向量搜索
- 混合搜索：同时使用关键词和向量搜索
- 智能路由：根据查询特征自动选择

### 4.4 结果融合器

**职责**：
- 融合多个搜索引擎的结果
- 支持多种融合算法
- 支持自定义融合策略

**主要逻辑**：
1. 接收多个引擎的搜索结果
2. 根据配置选择融合算法
3. 执行融合算法
4. 返回融合后的结果

**融合算法**：
- RRF（Reciprocal Rank Fusion）
- 加权融合
- 学习型融合

---

## 五、主要逻辑流程

### 5.1 文档索引流程

```
1. 接收文档索引请求
2. 生成文档向量（调用嵌入模型）
3. 构建 Qdrant Point（ID + 向量 + Payload）
4. 调用 Qdrant Upsert API
5. 等待 Qdrant 确认
6. 返回成功响应
```

### 5.2 向量搜索流程

```
1. 接收搜索请求
2. 生成查询向量（调用嵌入模型）
3. 检查缓存是否存在
4. 如果缓存命中，返回缓存结果
5. 如果缓存未命中
   - 构建 Qdrant 搜索请求
   - 添加过滤条件（如果有）
   - 调用 Qdrant Search API
   - 处理搜索结果
   - 缓存查询结果
6. 返回搜索结果
```

### 5.3 混合搜索流程

```
1. 接收搜索请求
2. 生成查询向量
3. 并行执行
   - 关键词搜索（FlexSearch/BM25）
   - 向量搜索（Qdrant）
4. 收集所有搜索结果
5. 使用融合算法合并结果
6. 返回融合后的结果
```

### 5.4 批量索引流程

```
1. 接收批量索引请求
2. 为每个文档生成向量
3. 构建 Qdrant Batch Upsert 请求
4. 调用 Qdrant Batch API
5. 等待 Qdrant 确认
6. 返回成功响应
```

---

## 六、技术选型

### 6.1 核心依赖

| 依赖 | 版本 | 用途 |
|------|------|------|
| Qdrant | latest | 向量数据库 |
| Redis | 7-alpine | 缓存 |
| gRPC | latest | 服务间通信 |
| Prometheus | latest | 监控指标 |
| Zap | latest | 日志系统 |

### 6.2 Qdrant 客户端

**Go 客户端**：
```go
import "github.com/qdrant/go-client/qdrant"

client, err := qdrant.NewClient(&qdrant.Config{
    Host: "http://qdrant:6333",
})
```

**主要 API**：
- `CreateCollection`：创建集合
- `Upsert`：添加/更新向量
- `Search`：向量搜索
- `Delete`：删除向量
- `Scroll`：遍历向量
- `Query`：复杂查询

### 6.3 依赖说明

**Qdrant**：
- 高性能向量搜索引擎
- 支持 HNSW 索引
- 支持混合搜索
- 支持过滤和聚合
- 支持向量量化
- 支持 SIMD 加速

**Redis**：
- 高性能键值存储
- 支持缓存和持久化
- 支持分布式场景
- 支持多种数据结构

**gRPC**：
- 高性能 RPC 框架
- 支持 Protocol Buffers
- 类型安全的通信

**Prometheus**：
- 监控指标收集
- 支持多种指标类型
- 支持指标导出

**Zap**：
- 结构化日志
- 高性能的日志记录
- 支持多种输出格式

---

## 七、设计原则

### 7.1 架构原则

1. **简化架构**：减少微服务数量，降低复杂度
2. **复用成熟方案**：使用 Qdrant 而不是自己实现
3. **性能优先**：利用 Qdrant 的优化和加速
4. **可扩展性**：支持水平扩展
5. **可观测性**：完善的日志、监控和追踪

### 7.2 代码原则

1. **简洁性**：代码简洁明了，易于理解和维护
2. **可测试性**：代码易于测试，单元测试覆盖率高
3. **模块化**：每个模块职责单一，高内聚低耦合
4. **错误处理**：完善的错误处理和恢复机制
5. **文档完善**：提供详细的 API 文档和使用示例

---

## 八、部署配置

### 8.1 Docker Compose 配置

```yaml
version: '3.8'

services:
  coordinator:
    build: .
    container_name: flexsearch-coordinator
    ports:
      - "50052:50052"
      - "9090:9090"
    environment:
      - LOG_LEVEL=info
      - QDRANT_URL=http://qdrant:6333
      - REDIS_URL=redis://redis:6379
    volumes:
      - ./configs:/root/configs
    depends_on:
      - redis
      - qdrant
    networks:
      - flexsearch-network
    restart: unless-stopped

  redis:
    image: redis:7-alpine
    container_name: flexsearch-redis
    ports:
      - "6379:6379"
    networks:
      - flexsearch-network
    restart: unless-stopped

  qdrant:
    image: qdrant/qdrant:latest
    container_name: flexsearch-qdrant
    ports:
      - "6333:6333"
      - "6334:6334"
    volumes:
      - qdrant_data:/qdrant/storage
    environment:
      - QDRANT__SERVICE__GRPC_PORT=6334
      - QDRANT__LOG_LEVEL=INFO
    networks:
      - flexsearch-network
    restart: unless-stopped

networks:
  flexsearch-network:
    driver: bridge

volumes:
  qdrant_data:
    driver: local
```

### 8.2 配置文件

```yaml
# configs/config.yaml

server:
  port: 50052
  metrics_port: 9090

log:
  level: info
  format: json

engines:
  flexsearch:
    enabled: true
    address: "inversearch:50051"
  
  bm25:
    enabled: true
    address: "bm25:50053"
  
  vector:
    enabled: true
    qdrant_url: "http://qdrant:6333"
    collections:
      default:
        dimension: 768
        distance: "Cosine"
        hnsw_config:
          m: 16
          ef_construct: 200
          full_scan_threshold: 10000
        optimizers_config:
          indexing_threshold: 10000

cache:
  redis:
    address: "redis:6379"
    ttl: 300
    max_size: 10000

router:
  strategy: "hybrid"
  weights:
    flexsearch: 0.3
    bm25: 0.3
    vector: 0.4

merger:
  algorithm: "rrf"
  rrf_k: 60
```

---

## 九、优势对比

### 9.1 方案对比

| 维度 | 独立向量搜索服务 | 集成 Qdrant |
|------|-----------------|-------------|
| **开发时间** | 3-6 个月 | 1-2 周 |
| **代码量** | ~5000 行 | ~500 行 |
| **微服务数量** | 3 个 | 2 个 |
| **维护成本** | 高 | 低 |
| **性能** | 需要验证 | 经过充分优化 |
| **功能完整性** | 需要逐步实现 | 功能完善 |
| **架构复杂度** | 高 | 低 |
| **部署复杂度** | 高 | 低 |
| **故障风险** | 高 | 低 |
| **社区支持** | 无 | 活跃 |

### 9.2 核心优势

1. **开发效率**：从 3-6 个月缩短到 1-2 周
2. **架构简化**：减少一个微服务，降低复杂度
3. **性能保证**：使用 Qdrant 的优化和加速
4. **功能完善**：开箱即用，无需逐步实现
5. **维护简单**：由 Qdrant 社区维护
6. **成本降低**：开发和维护成本大幅降低

---

## 十、实施计划

### 10.1 开发阶段

| 阶段 | 任务 | 时间 |
|------|------|------|
| **阶段 1** | 集成 Qdrant 客户端 | 3 天 |
| **阶段 2** | 实现向量索引功能 | 2 天 |
| **阶段 3** | 实现向量搜索功能 | 2 天 |
| **阶段 4** | 实现缓存管理 | 2 天 |
| **阶段 5** | 实现混合搜索 | 3 天 |
| **阶段 6** | 测试和优化 | 3 天 |

### 10.2 测试阶段

| 测试类型 | 内容 |
|---------|------|
| **单元测试** | 测试各个模块的功能 |
| **集成测试** | 测试与 Qdrant 的集成 |
| **性能测试** | 测试搜索性能和吞吐量 |
| **压力测试** | 测试高并发场景 |
| **故障测试** | 测试故障恢复能力 |

---

## 十一、监控和运维

### 11.1 监控指标

| 指标 | 说明 |
|------|------|
| **搜索延迟** | 搜索请求的响应时间 |
| **搜索吞吐量** | 每秒处理的搜索请求数 |
| **缓存命中率** | 缓存命中的比例 |
| **Qdrant 延迟** | Qdrant API 调用的延迟 |
| **错误率** | 错误请求的比例 |

### 11.2 日志记录

| 日志类型 | 内容 |
|---------|------|
| **请求日志** | 记录所有搜索请求 |
| **响应日志** | 记录所有搜索响应 |
| **错误日志** | 记录所有错误信息 |
| **性能日志** | 记录性能指标 |

### 11.3 告警规则

| 告警 | 条件 |
|------|------|
| **高延迟** | 搜索延迟 > 1s |
| **高错误率** | 错误率 > 5% |
| **低缓存命中率** | 缓存命中率 < 50% |
| **Qdrant 不可用** | Qdrant 健康检查失败 |

---

## 十二、总结

通过直接集成 Qdrant 向量数据库，我们能够：

1. ✅ **大幅降低开发成本**：从 3-6 个月缩短到 1-2 周
2. ✅ **简化架构**：减少一个微服务，降低复杂度
3. ✅ **保证性能**：使用 Qdrant 的优化和加速
4. ✅ **功能完善**：开箱即用，无需逐步实现
5. ✅ **降低维护成本**：由 Qdrant 社区维护
6. ✅ **提高稳定性**：使用经过充分验证的成熟方案

这种方案避免了重复造轮子，充分利用了 Qdrant 的优势，是一个更加务实和高效的选择。
