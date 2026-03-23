# 查询协调器模块分阶段执行方案

## 阶段一：基础框架搭建（第1–2周）

### 目标
建立查询协调器的基础架构，实现基本的服务框架和配置管理。

### 任务清单

#### 1.1 项目初始化
- 创建 `services/coordinator/` 目录结构  
- 初始化 Go 模块（`go mod init`）  
- 配置基础依赖：
  - gRPC
  - Zap（日志）
  - Viper（配置）
  - Prometheus（监控）  
- 编写 `Dockerfile` 和 `docker-compose.yml` 配置文件  

#### 1.2 配置管理
- 实现 `internal/config/config.go` —— 配置结构定义  
- 实现 `internal/config/engines.go` —— 搜索引擎配置管理  
- 创建 `configs/config.yaml` —— 配置文件模板  
- 实现配置热加载功能（基于文件监听或 API 触发）

#### 1.3 日志与监控
- 实现 `internal/util/logger.go` —— 封装 Zap 日志工具  
- 实现 `internal/util/metrics.go` —— Prometheus 指标收集（请求量、延迟等）  
- 实现 `internal/util/error.go` —— 统一错误处理与封装工具

#### 1.4 基础服务框架
- 创建 `cmd/main.go` —— 服务主入口  
- 实现服务启动流程与优雅关闭机制（信号捕获）  
- 提供健康检查接口 `/healthz`（HTTP + gRPC）  
- 配置并启动 gRPC 服务器（默认端口：50051）

### 交付物
- 可运行的查询协调器服务框架  
- 完整的配置管理系统（支持 YAML 配置 + 热更新）  
- 基础日志与监控能力（结构化日志 + 指标暴露）  

---

## 阶段二：数据模型和协议定义（第3周）

### 目标
定义查询协调器的核心数据模型及 gRPC 接口协议。

### 任务清单

#### 2.1 数据模型
- 实现 `internal/model/request.go`
  - `SearchRequest`: 搜索请求主体
  - `QueryInfo`: 查询上下文信息
  - `EngineConfig`: 引擎个性化配置
- 实现 `internal/model/response.go`
  - `SearchResponse`: 综合响应体
  - `SearchResult`: 单个结果项
  - `EngineResult`: 各引擎返回的原始结果

#### 2.2 gRPC 协议定义
- 创建 `proto/coordinator.proto`
  - 定义 `SearchService` 服务：
    - `rpc Search(SearchRequest) returns (SearchResponse)`
    - `rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse)`
  - 定义所有消息类型（message）
- 使用 `protoc` 生成 Go 代码（含 gRPC Stub）
- 实现 gRPC Server 接口骨架（空实现占位）

#### 2.3 单元测试
- 编写数据模型单元测试（序列化/反序列化、字段验证）
- 测试 proto 到内部模型的转换逻辑
- 覆盖边界条件与默认值处理

### 交付物
- 完整的数据模型定义（Go + Proto）
- 标准化的 gRPC 协议文件与生成代码
- 单元测试覆盖率 ≥80%

---

## 阶段三：搜索引擎客户端（第4–5周）

### 目标
实现与各搜索引擎的 gRPC 客户端连接，支持高可用通信。

### 任务清单

#### 3.1 客户端接口
- 实现 `internal/engine/client.go`
  - 定义 `EngineClient` 接口：
    - `Connect() error`
    - `Disconnect() error`
    - `Search(ctx context.Context, req *SearchRequest) (*EngineResult, error)`
    - `HealthCheck() bool`

#### 3.2 FlexSearch 客户端
- 实现 `internal/engine/flexsearch.go`
  - gRPC 连接池管理（使用 `grpc.ClientConnPool` 或自研）
  - 请求封装与响应解析
  - 支持重试机制（指数退避）
  - 超时控制与熔断保护

#### 3.3 BM25 客户端
- 实现 `internal/engine/bm25.go`
  - 连接管理与健康探测
  - 搜索请求适配（字段映射）
  - 响应标准化处理
  - 错误码映射与本地兜底

#### 3.4 向量搜索客户端
- 实现 `internal/engine/vector.go`
  - 支持向量嵌入传递（Embedding 字段）
  - 处理相似度得分归一化
  - 并行调用优化（批处理支持预留）

#### 3.5 集成测试
- 编写模拟服务进行客户端集成测试
- 验证连接池行为、超时、断线重连
- 模拟网络异常下的重试与降级策略

### 交付物
- 四大搜索引擎客户端完整实现（FlexSearch / BM25 / Vector / 兜底）
- 统一的连接池与重试机制
- 集成测试报告（包含性能与稳定性指标）

---

## 阶段四：查询路由器（第6–7周）

### 目标
实现智能路由决策系统，根据查询特征选择最优搜索引擎组合。

### 任务清单

#### 4.1 路由决策核心
- 实现 `internal/router/router.go`
  - 分析查询关键词、长度、语义倾向
  - 输出目标引擎列表（单引擎 or 多引擎并行）
  - 支持动态权重调整（基于历史表现）

#### 4.2 路由策略
- 实现 `internal/router/strategy.go`
  - **精确匹配策略**：短词、术语 → BM25
  - **模糊搜索策略**：拼写容错 → FlexSearch
  - **语义搜索策略**：长句、自然语言 → 向量引擎
  - **混合搜索策略**：多引擎协同触发
  - **自动路由策略**：AI 模型推荐（未来扩展点）

#### 4.3 查询优化器
- 实现 `internal/router/optimizer.go`
  - 查询重写（同义词扩展、停用词过滤）
  - 查询建议生成（用于前端提示）
  - 性能统计埋点（各策略命中率、耗时）

#### 4.4 单元测试
- 编写典型场景测试用例（如“新冠疫苗”、“how to fix…”）
- 验证策略切换逻辑正确性
- 测试查询改写效果与安全性（防注入）

### 交付物
- 可插拔的路由策略体系
- 查询分析与优化能力
- 单元测试覆盖主流查询模式

---

## 阶段五：结果融合器（第8–9周）

### 目标
实现多源搜索结果的融合排序，输出统一高质量结果集。

### 任务清单

#### 5.1 融合接口
- 实现 `internal/merger/merger.go`
  - 定义 `Merger` 接口：
    - `Merge(results map[string]*EngineResult) *SearchResponse`
    - `Sort(results []*SearchResult)`
    - `Deduplicate(results []*SearchResult)`

#### 5.2 RRF 融合策略
- 实现 `internal/merger/rrf.go` —— Reciprocal Rank Fusion
  - 支持参数 `k` 可配置（默认 60）
  - 计算公式：`score = Σ(1/(k + rank_i))`
  - 保留 Top-K 结果

#### 5.3 加权融合策略
- 实现 `internal/merger/weighted.go`
  - 支持按引擎设置静态权重（如：vector: 0.6, bm25: 0.3, flex: 0.1）
  - 归一化各引擎得分后加权求和
  - 支持动态权重反馈调节（后续迭代）

#### 5.4 单元测试
- 构造多组模拟结果测试 RRF 效果
- 验证加权融合准确性
- 测试去重逻辑（基于 ID 或 URL）

### 交付物
- 结果融合核心模块完成
- 支持两种主流融合算法（RRF + 加权）
- 单元测试验证融合质量一致性

---

## 阶段六：缓存管理器（第10周）

### 目标
引入缓存机制，提升高频查询响应速度，降低后端负载。

### 任务清单

#### 6.1 缓存接口
- 实现 `internal/cache/cache.go`
  - 定义通用 `Cache` 接口：
    - `Get(key string) ([]byte, bool)`
    - `Set(key string, value []byte, ttl time.Duration)`
    - `Delete(key string)`
    - `Clear()`

#### 6.2 Redis 缓存实现
- 实现 `internal/cache/redis.go`
  - 使用 `go-redis` 连接 Redis 集群
  - 自动生成缓存键（MD5(SearchRequest)）
  - 支持 TTL 配置（默认 5 分钟）
  - 实现 LRU 淘汰策略（通过 Redis 配置驱动）

#### 6.3 缓存统计与管理
- 实现缓存命中率统计（Prometheus 暴露 `cache_hits_total`, `cache_misses_total`）
- 提供缓存预热接口（Admin API）
- 实现批量清理功能（按前缀或全量）

#### 6.4 单元测试
- Mock Redis 测试读写逻辑
- 验证过期机制与并发安全
- 测试缓存穿透防护（空值缓存）

### 交付物
- 完整的缓存管理模块
- Redis 集成与生产级配置
- 缓存命中监控与运维工具

---

## 阶段七：核心查询流程（第11–12周）

### 目标
整合全部模块，构建端到端的高性能查询处理流水线。

### 任务清单

#### 7.1 查询处理服务
- 实现主处理逻辑 `service/search.go`
  - 接收 gRPC 请求
  - 缓存检查（命中则直接返回）
  - 路由决策（确定调用哪些引擎）
  - 并行发起引擎调用
  - 执行结果融合
  - 更新缓存（异步写入）
  - 返回最终响应

#### 7.2 并行执行机制
- 使用 Goroutine 并发调用多个引擎
- 使用 `sync.WaitGroup` 控制协程同步
- 设置整体超时（context.WithTimeout，默认 800ms）
- 支持部分失败容忍（至少一个成功即响应）

#### 7.3 超时与容错
- 实现精细化超时控制（总超时 vs 子请求超时）
- 引擎故障检测与自动剔除（临时隔离）
- 降级策略：
  - 主引擎失败 → 切换备用引擎
  - 全部失败 → 返回缓存近似结果或空结果
- 重试机制（仅对幂等操作启用）

#### 7.4 集成测试
- 编写端到端测试用例（E2E）
  - 正常路径：查询 → 缓存未命中 → 路由 → 并行调用 → 融合 → 缓存写入
  - 异常路径：某引擎宕机、超时、返回空
- 使用 `testcontainers-go` 搭建本地测试环境
- 压力测试：模拟 QPS=100 场景下稳定性

### 交付物
- 完整的查询处理闭环
- 高性能并行调度能力
- 成熟的容错与降级机制
- E2E 测试报告与性能基线

---

## 附录：总体进度计划表（甘特图概览）

| 阶段 | 时间范围 | 关键里程碑 |
|------|--------|-----------|
| 一、基础框架搭建 | 第1–2周 | 服务可启动，配置热加载可用 |
| 二、数据模型与协议 | 第3周 | Proto 定义冻结，代码生成完成 |
| 三、搜索引擎客户端 | 第4–5周 | 所有客户端通过集成测试 |
| 四、查询路由器 | 第6–7周 | 路由策略上线，支持自动决策 |
| 五、结果融合器 | 第8–9周 | RRF 与加权融合稳定运行 |
| 六、缓存管理器 | 第10周 | Redis 缓存上线，命中率 >40% |
| 七、核心流程整合 | 第11–12周 | E2E 测试通过，准备上线 |
