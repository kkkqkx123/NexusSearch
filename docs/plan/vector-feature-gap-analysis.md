# 向量客户端功能对比分析

## 一、功能对比总览

### 1.1 VectorEngine 接口对比

| 功能分类 | 功能名称 | Rust ref 实现 | Go 当前实现 | 状态 |
|---------|---------|--------------|------------|------|
| **集合管理** | create_collection | ✅ | ✅ | 已实现 |
| | delete_collection | ✅ | ✅ | 已实现 |
| | collection_exists | ✅ | ❌ | **需补充** |
| | collection_info | ✅ | ❌ | **需补充** |
| **向量点操作** | upsert | ✅ | ✅ | 已实现 |
| | upsert_batch | ✅ | ✅ | 已实现 |
| | get | ✅ | ❌ | **需补充** |
| | get_batch | ✅ | ❌ | **需补充** |
| | delete | ✅ | ✅ | 已实现 |
| | delete_batch | ✅ | ❌ | **需补充** |
| | delete_by_filter | ✅ | ❌ | **需补充** |
| **搜索功能** | search | ✅ | ✅ | 已实现 |
| | search_batch | ✅ | ❌ | **需补充** |
| **统计功能** | count | ✅ | ❌ | **需补充** |
| **Payload 操作** | set_payload | ✅ | ❌ | **需补充** |
| | delete_payload | ✅ | ❌ | **需补充** |
| **Scroll 操作** | scroll | ✅ | ❌ | **需补充** |
| **索引管理** | create_payload_index | ✅ | ❌ | **需补充** |
| | delete_payload_index | ✅ | ❌ | **需补充** |
| | list_payload_indexes | ✅ | ❌ | **需补充** |
| **健康检查** | health_check | ✅ | ✅ | 已实现 |

### 1.2 实现完成度

- **已实现**: 8/21 (38%)
- **需补充**: 13/21 (62%)

---

## 二、需补充功能详细分析

### 2.1 集合管理功能

#### collection_exists
```go
func (c *VectorClient) CollectionExists(ctx context.Context, name string) (bool, error)
```
**用途**: 检查集合是否存在，用于初始化时自动创建集合

#### collection_info
```go
type CollectionInfo struct {
    Name              string
    VectorCount       uint64
    IndexedVectorCount uint64
    PointsCount       uint64
    SegmentsCount     uint64
    Config            CollectionConfig
    Status            CollectionStatus
}

func (c *VectorClient) CollectionInfo(ctx context.Context, name string) (*CollectionInfo, error)
```
**用途**: 获取集合详细信息，用于监控和调试

### 2.2 向量点操作功能

#### get / get_batch
```go
func (c *VectorClient) Get(ctx context.Context, collection string, id string) (*VectorPoint, error)
func (c *VectorClient) GetBatch(ctx context.Context, collection string, ids []string) ([]*VectorPoint, error)
```
**用途**: 根据 ID 获取向量点，用于数据验证和调试

#### delete_batch
```go
func (c *VectorClient) DeleteBatch(ctx context.Context, collection string, ids []string) error
```
**用途**: 批量删除向量点，提高删除效率

#### delete_by_filter
```go
func (c *VectorClient) DeleteByFilter(ctx context.Context, collection string, filter *VectorFilter) error
```
**用途**: 按条件批量删除，例如删除特定类别的文档

### 2.3 搜索功能增强

#### search_batch
```go
func (c *VectorClient) SearchBatch(ctx context.Context, collection string, queries []*SearchQuery) ([][]*SearchResult, error)
```
**用途**: 批量搜索，减少网络往返，提高吞吐量

#### SearchQuery 增强
当前 SearchQuery 缺少以下参数：
- `offset`: 分页偏移
- `filter`: 过滤条件
- `with_vector`: 是否返回向量

### 2.4 统计功能

#### count
```go
func (c *VectorClient) Count(ctx context.Context, collection string) (uint64, error)
```
**用途**: 统计集合中的向量点数量

### 2.5 Payload 操作

#### set_payload / delete_payload
```go
func (c *VectorClient) SetPayload(ctx context.Context, collection string, ids []string, payload map[string]interface{}) error
func (c *VectorClient) DeletePayload(ctx context.Context, collection string, ids []string, keys []string) error
```
**用途**: 动态更新向量点的元数据，无需重新索引

### 2.6 Scroll 操作

#### scroll
```go
func (c *VectorClient) Scroll(ctx context.Context, collection string, limit int, offset string, withPayload, withVector bool) ([]*VectorPoint, string, error)
```
**用途**: 分页遍历集合中的所有点，用于数据迁移和批量处理

### 2.7 索引管理

#### create_payload_index / delete_payload_index / list_payload_indexes
```go
func (c *VectorClient) CreatePayloadIndex(ctx context.Context, collection string, field string, schemaType PayloadSchemaType) error
func (c *VectorClient) DeletePayloadIndex(ctx context.Context, collection string, field string) error
func (c *VectorClient) ListPayloadIndexes(ctx context.Context, collection string) ([]PayloadIndexInfo, error)
```
**用途**: 管理 Payload 字段索引，加速过滤查询

---

## 三、类型定义对比

### 3.1 过滤器系统 (VectorFilter)

**Rust 实现的完整过滤器类型**:

| 类型 | 说明 | Go 实现状态 |
|-----|------|------------|
| Match | 精确匹配 | ❌ |
| MatchAny | 匹配任意值 | ❌ |
| Range | 范围查询 (gt, gte, lt, lte) | ❌ |
| IsEmpty | 空值检查 | ❌ |
| IsNull | Null 检查 | ❌ |
| HasId | ID 过滤 | ❌ |
| GeoRadius | 地理半径查询 | ❌ |
| GeoBoundingBox | 地理边界框查询 | ❌ |
| ValuesCount | 数组长度过滤 | ❌ |
| Contains | 数组包含 | ❌ |
| Nested | 嵌套过滤 | ❌ |

**建议**: 实现完整的 VectorFilter 类型系统

### 3.2 配置类型

| 类型 | 说明 | Go 实现状态 |
|-----|------|------------|
| CollectionConfig | 集合配置 | 部分 |
| HnswConfig | HNSW 索引配置 | ❌ |
| QuantizationConfig | 量化配置 | ❌ |
| DistanceMetric | 距离度量 | ✅ |

### 3.3 结果类型

| 类型 | 说明 | Go 实现状态 |
|-----|------|------------|
| VectorPoint | 向量点 | ✅ |
| SearchResult | 搜索结果 | ✅ |
| UpsertResult | 写入结果 | ❌ |
| DeleteResult | 删除结果 | ❌ |
| CollectionInfo | 集合信息 | ❌ |
| HealthStatus | 健康状态 | ❌ |

---

## 四、优先级建议

### P0 - 核心功能（必须实现）
1. **collection_exists** - 初始化检查
2. **count** - 监控统计
3. **get / get_batch** - 数据查询
4. **VectorFilter** - 过滤查询支持

### P1 - 重要功能（建议实现）
5. **delete_batch** - 批量删除
6. **delete_by_filter** - 条件删除
7. **search_batch** - 批量搜索
8. **scroll** - 数据遍历

### P2 - 增强功能（可选实现）
9. **collection_info** - 详细信息
10. **set_payload / delete_payload** - Payload 更新
11. **create_payload_index / delete_payload_index** - 索引管理
12. **list_payload_indexes** - 索引列表

---

## 五、实现建议

### 5.1 文件结构调整

建议创建以下新文件：

```
coordinator/internal/engine/
├── vector.go           # 主客户端（已有，需增强）
├── vector_filter.go    # 过滤器类型定义（新增）
├── vector_types.go     # 类型定义（新增）
└── vector_utils.go     # 工具函数（新增）
```

### 5.2 接口设计

```go
type VectorClientInterface interface {
    // 集合管理
    CreateCollection(ctx context.Context, name string, config *CollectionConfig) error
    DeleteCollection(ctx context.Context, name string) error
    CollectionExists(ctx context.Context, name string) (bool, error)
    CollectionInfo(ctx context.Context, name string) (*CollectionInfo, error)
    
    // 向量操作
    Upsert(ctx context.Context, collection string, point *VectorPoint) error
    UpsertBatch(ctx context.Context, collection string, points []*VectorPoint) error
    Get(ctx context.Context, collection string, id string) (*VectorPoint, error)
    GetBatch(ctx context.Context, collection string, ids []string) ([]*VectorPoint, error)
    Delete(ctx context.Context, collection string, id string) error
    DeleteBatch(ctx context.Context, collection string, ids []string) error
    DeleteByFilter(ctx context.Context, collection string, filter *VectorFilter) error
    
    // 搜索
    Search(ctx context.Context, collection string, query *SearchQuery) ([]*SearchResult, error)
    SearchBatch(ctx context.Context, collection string, queries []*SearchQuery) ([][]*SearchResult, error)
    
    // 统计
    Count(ctx context.Context, collection string) (uint64, error)
    
    // Payload
    SetPayload(ctx context.Context, collection string, ids []string, payload map[string]interface{}) error
    DeletePayload(ctx context.Context, collection string, ids []string, keys []string) error
    
    // Scroll
    Scroll(ctx context.Context, collection string, opts *ScrollOptions) ([]*VectorPoint, string, error)
    
    // 索引
    CreatePayloadIndex(ctx context.Context, collection string, field string, schemaType PayloadSchemaType) error
    DeletePayloadIndex(ctx context.Context, collection string, field string) error
    
    // 健康
    HealthCheck(ctx context.Context) bool
}
```

### 5.3 与 EngineClient 接口的关系

当前 `VectorClient` 实现了 `EngineClient` 接口用于搜索协调。建议：

1. 保持 `EngineClient` 接口不变（仅包含 Search 方法）
2. 扩展 `VectorClient` 添加上述完整功能
3. 在 coordinator 中需要时调用扩展方法

---

## 六、参考实现位置

| 功能 | Rust 参考文件 |
|-----|-------------|
| VectorEngine trait | `ref/vector-client/src/engine/mod.rs` |
| Qdrant 实现 | `ref/vector-client/src/engine/qdrant/mod.rs` |
| 过滤器转换 | `ref/vector-client/src/engine/qdrant/filter.rs` |
| 工具函数 | `ref/vector-client/src/engine/qdrant/utils.rs` |
| 类型定义 | `ref/vector-client/src/types/*.rs` |
| API 封装 | `ref/vector-client/src/api/embedded/core.rs` |
