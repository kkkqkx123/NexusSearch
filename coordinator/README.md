# FlexSearch Query Coordinator

查询协调器是 FlexSearch 中间层的核心组件，负责接收来自 API 网关的查询请求，根据查询类型路由到不同的搜索引擎，并协调多个搜索引擎的并行执行和结果融合。

## 功能特性

- 智能查询路由：根据查询特征自动选择合适的搜索引擎
- 并行执行：同时调用多个搜索引擎，提高响应速度
- 结果融合：支持 RRF、加权等多种融合策略
- 查询缓存：基于 Redis 的查询结果缓存
- 超时控制：支持查询超时和引擎超时控制
- 错误处理：完善的错误处理和降级机制
- 监控指标：集成 Prometheus 监控指标
- 健康检查：支持 gRPC 健康检查

## 技术栈

- Go 1.21
- gRPC
- Redis
- Zap (日志)
- Prometheus (监控)
- Viper (配置管理)

## 目录结构

```
services/coordinator/
├── cmd/
│   └── main.go              # 入口文件
├── internal/
│   ├── config/              # 配置管理
│   ├── router/              # 查询路由器
│   ├── merger/              # 结果融合器
│   ├── engine/              # 搜索引擎客户端
│   ├── cache/               # 缓存管理
│   ├── model/               # 数据模型
│   └── util/                # 工具函数
├── proto/                   # gRPC 协议定义
├── configs/                 # 配置文件
├── Dockerfile
├── docker-compose.yml
└── README.md
```

## 快速开始

### 本地运行

1. 安装依赖：
```bash
go mod download
```

2. 配置服务：
编辑 `configs/config.yaml` 文件，配置搜索引擎地址和其他参数。

3. 运行服务：
```bash
go run cmd/main.go
```

### Docker 运行

1. 构建镜像：
```bash
docker build -t flexsearch-coordinator .
```

2. 运行容器：
```bash
docker-compose up -d
```

## 配置说明

主要配置项：

- `server`: 服务配置
- `grpc`: gRPC 服务配置
- `redis`: Redis 缓存配置
- `engines`: 搜索引擎配置
- `cache`: 缓存策略配置
- `metrics`: 监控指标配置
- `logging`: 日志配置

详细配置说明请参考 `configs/config.yaml`。

## API 文档

### gRPC 服务

#### Search

搜索接口，接收查询请求并返回搜索结果。

#### HealthCheck

健康检查接口，用于监控服务状态。

## 监控指标

服务提供以下 Prometheus 指标：

- `grpc_requests_total`: gRPC 请求总数
- `grpc_request_duration_seconds`: gRPC 请求延迟
- `query_latency_seconds`: 查询延迟
- `engine_latency_seconds`: 搜索引擎延迟
- `cache_hits_total`: 缓存命中数
- `cache_misses_total`: 缓存未命中数
- `errors_total`: 错误总数

指标访问地址：`http://localhost:9090/metrics`

## 开发指南

### 添加新的搜索引擎

1. 在 `internal/engine/` 目录下创建新的客户端实现
2. 实现 `EngineClient` 接口
3. 在配置文件中添加引擎配置
4. 在路由器中添加路由逻辑

### 添加新的融合策略

1. 在 `internal/merger/` 目录下创建新的融合策略实现
2. 实现 `Merger` 接口
3. 在配置文件中添加策略配置

## 许可证

MIT License
