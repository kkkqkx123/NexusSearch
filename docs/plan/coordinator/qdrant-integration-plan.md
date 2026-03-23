# Qdrant 集成分阶段实施方案

## 一、背景与目标

### 1.1 当前状态

查询协调器目前包含一个模拟的向量搜索客户端（`internal/engine/vector.go`），该实现：
- 使用模拟的向量嵌入生成（基于 MD5 哈希）
- 使用模拟的文档向量（基于正弦函数）
- 自己实现余弦相似度计算
- 不连接任何真实的向量数据库

### 1.2 目标

将模拟的向量搜索客户端替换为真正的 Qdrant 集成，实现：
- 连接到 Qdrant 向量数据库
- 使用真实的向量嵌入（通过嵌入模型服务）
- 利用 Qdrant 的 HNSW 索引和优化
- 支持混合搜索（密集向量 + 稀疏向量）
- 保持与现有接口的兼容性

### 1.3 预期收益

| 收益项 | 说明 |
|--------|------|
| **性能提升** | 利用 Qdrant 的 SIMD 加速和优化 |
| **功能完善** | 支持混合搜索、过滤、聚合等高级功能 |
| **维护简化** | 由 Qdrant 社区维护核心功能 |
| **扩展性** | 支持分布式部署和水平扩展 |

---

## 二、架构变更

### 2.1 当前架构

```
查询协调器
  └── VectorClient (模拟)
       ├── 模拟向量生成
       ├── 模拟文档向量
       └── 自己计算相似度
```

### 2.2 目标架构

```
查询协调器
  └── VectorClient (Qdrant)
       ├── Qdrant 客户端连接
       ├── 嵌入模型服务调用
       ├── Qdrant API 调用
       └── 结果处理和缓存
```

### 2.3 依赖服务

新增依赖：
- **Qdrant 服务**：向量数据库（Docker 容器）
- **嵌入模型服务**：生成文本向量（可选，可使用外部 API）

---

## 三、分阶段实施方案

## 阶段一：Qdrant 环境准备（第1-2天）

### 目标
搭建 Qdrant 开发环境，验证基本功能。

### 任务清单

#### 1.1 Docker Compose 配置更新

**文件**：`services/coordinator/docker-compose.yml`

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
      - QDRANT__SERVICE__MAX_REQUEST_SIZE_MB=32
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

#### 1.2 配置文件更新

**文件**：`services/coordinator/configs/config.yaml`

```yaml
engines:
  vector:
    enabled: true
    qdrant_url: "http://qdrant:6333"
    grpc_url: "qdrant:6334"
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
          max_segment_size: 200000
          memmap_threshold: 20000
        quantization_config:
          scalar:
            type: "int8"
            always_ram: true
    timeout: 5s
    max_retries: 3
    cache_ttl: 300
```

#### 1.3 Go 依赖添加

**文件**：`services/coordinator/go.mod`

```go
require (
    github.com/qdrant/go-client/qdrant v1.7.0
)
```

执行命令：
```bash
cd services/coordinator
go get github.com/qdrant/go-client/qdrant@latest
go mod tidy
```

#### 1.4 启动测试

```bash
cd services/coordinator
docker-compose up -d qdrant
```

验证 Qdrant 服务：
```bash
curl http://localhost:6333/
curl http://localhost:6333/collections
```

### 验收标准
- ✅ Qdrant 容器成功启动
- ✅ HTTP API 可访问
- ✅ 配置文件正确加载
- ✅ Go 依赖正确安装

---

## 阶段二：Qdrant 客户端基础封装（第3-4天）

### 目标
实现 Qdrant 客户端的基础连接和集合管理功能。

### 任务清单

#### 2.1 配置结构更新

**文件**：`services/coordinator/internal/config/engines.go`

```go
package config

type VectorEngineConfig struct {
    Enabled     bool                    `yaml:"enabled"`
    QdrantURL   string                  `yaml:"qdrant_url"`
    GRPCURL     string                  `yaml:"grpc_url"`
    Collections map[string]CollectionConfig `yaml:"collections"`
    Timeout     time.Duration           `yaml:"timeout"`
    MaxRetries  int                     `yaml:"max_retries"`
    CacheTTL    int                     `yaml:"cache_ttl"`
}

type CollectionConfig struct {
    Dimension          int                    `yaml:"dimension"`
    Distance           string                 `yaml:"distance"`
    HNSWConfig         *HNSWConfig            `yaml:"hnsw_config"`
    OptimizersConfig   *OptimizersConfig      `yaml:"optimizers_config"`
    QuantizationConfig *QuantizationConfig    `yaml:"quantization_config"`
}

type HNSWConfig struct {
    M                  int     `yaml:"m"`
    EfConstruct        int     `yaml:"ef_construct"`
    FullScanThreshold  int     `yaml:"full_scan_threshold"`
}

type OptimizersConfig struct {
    IndexingThreshold int `yaml:"indexing_threshold"`
    MaxSegmentSize    int `yaml:"max_segment_size"`
    MemmapThreshold   int `yaml:"memmap_threshold"`
}

type QuantizationConfig struct {
    Scalar *ScalarQuantization `yaml:"scalar"`
}

type ScalarQuantization struct {
    Type      string `yaml:"type"`
    AlwaysRAM bool   `yaml:"always_ram"`
}
```

#### 2.2 Qdrant 客户端实现

**文件**：`services/coordinator/internal/engine/qdrant_client.go`（新建）

```go
package engine

import (
	"context"
	"fmt"
	"time"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/qdrant/go-client/qdrant"
)

type QdrantClient struct {
	config   *config.VectorEngineConfig
	client   *qdrant.Client
	logger   *util.Logger
	connPool *ConnectionPool
}

type ConnectionPool struct {
	clients []*qdrant.Client
	current int
}

func NewQdrantClient(cfg *config.VectorEngineConfig, logger *util.Logger) (*QdrantClient, error) {
	if cfg == nil {
		return nil, fmt.Errorf("vector config cannot be nil")
	}

	client, err := qdrant.NewClient(&qdrant.Config{
		Host: cfg.QdrantURL,
	})
	if err != nil {
		return nil, fmt.Errorf("failed to create Qdrant client: %w", err)
	}

	qc := &QdrantClient{
		config: cfg,
		client: client,
		logger: logger,
	}

	if err := qc.initializeCollections(context.Background()); err != nil {
		return nil, fmt.Errorf("failed to initialize collections: %w", err)
	}

	return qc, nil
}

func (qc *QdrantClient) initializeCollections(ctx context.Context) error {
	for name, cfg := range qc.config.Collections {
		if err := qc.createCollectionIfNotExists(ctx, name, cfg); err != nil {
			return fmt.Errorf("failed to create collection %s: %w", name, err)
		}
	}
	return nil
}

func (qc *QdrantClient) createCollectionIfNotExists(ctx context.Context, name string, cfg config.CollectionConfig) error {
	collections, err := qc.client.ListCollections(ctx)
	if err != nil {
		return fmt.Errorf("failed to list collections: %w", err)
	}

	for _, coll := range collections {
		if coll.Name == name {
			qc.logger.Infof("Collection %s already exists", name)
			return nil
		}
	}

	distance := qdrant.Distance_Cosine
	switch cfg.Distance {
	case "Euclid":
		distance = qdrant.Distance_Euclid
	case "Dot":
		distance = qdrant.Distance_Dot
	}

	createReq := &qdrant.CreateCollection{
		CollectionName: name,
		VectorsConfig: qdrant.NewVectorsConfig(&qdrant.VectorParams{
			Size:     uint64(cfg.Dimension),
			Distance: distance,
			HnswConfig: &qdrant.HnswConfigDiff{
				M:                uint32(cfg.HNSWConfig.M),
				EfConstruct:      uint32(cfg.HNSWConfig.EfConstruct),
				FullScanThreshold: uint32(cfg.HNSWConfig.FullScanThreshold),
			},
		}),
		OptimizersConfig: &qdrant.OptimizersConfigDiff{
			IndexingThreshold: uint64(cfg.OptimizersConfig.IndexingThreshold),
			MaxSegmentSize:    uint64(cfg.OptimizersConfig.MaxSegmentSize),
			MemmapThreshold:   uint64(cfg.OptimizersConfig.MemmapThreshold),
		},
	}

	if cfg.QuantizationConfig != nil && cfg.QuantizationConfig.Scalar != nil {
		createReq.QuantizationConfig = &qdrant.QuantizationConfig{
			Scalar: &qdrant.ScalarQuantization{
				Type:      qdrant.ScalarType_Int8,
				AlwaysRam: &cfg.QuantizationConfig.Scalar.AlwaysRAM,
			},
		}
	}

	if err := qc.client.CreateCollection(ctx, createReq); err != nil {
		return fmt.Errorf("failed to create collection: %w", err)
	}

	qc.logger.Infof("Collection %s created successfully", name)
	return nil
}

func (qc *QdrantClient) Close() error {
	if qc.client != nil {
		return qc.client.Close()
	}
	return nil
}

func (qc *QdrantClient) GetClient() *qdrant.Client {
	return qc.client
}

func (qc *QdrantClient) GetConfig() *config.VectorEngineConfig {
	return qc.config
}
```

#### 2.3 单元测试

**文件**：`services/coordinator/internal/engine/qdrant_client_test.go`（新建）

```go
package engine

import (
	"context"
	"testing"
	"time"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewQdrantClient(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"test": {
				Dimension: 128,
				Distance:  "Cosine",
				HNSWConfig: &config.HNSWConfig{
					M:                 16,
					EfConstruct:       200,
					FullScanThreshold: 10000,
				},
				OptimizersConfig: &config.OptimizersConfig{
					IndexingThreshold: 10000,
					MaxSegmentSize:    200000,
					MemmapThreshold:   20000,
				},
			},
		},
		Timeout:    5 * time.Second,
		MaxRetries: 3,
		CacheTTL:   300,
	}

	logger := util.NewLogger("info")
	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	require.NotNil(t, client)
	defer client.Close()

	assert.Equal(t, cfg, client.GetConfig())
	assert.NotNil(t, client.GetClient())
}

func TestQdrantClient_CreateCollection(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"test-create": {
				Dimension: 64,
				Distance:  "Cosine",
				HNSWConfig: &config.HNSWConfig{
					M:                 16,
					EfConstruct:       200,
					FullScanThreshold: 10000,
				},
				OptimizersConfig: &config.OptimizersConfig{
					IndexingThreshold: 10000,
					MaxSegmentSize:    200000,
					MemmapThreshold:   20000,
				},
			},
		},
	}

	logger := util.NewLogger("info")
	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	ctx := context.Background()

	collections, err := client.GetClient().ListCollections(ctx)
	require.NoError(t, err)

	found := false
	for _, coll := range collections {
		if coll.Name == "test-create" {
			found = true
			break
		}
	}
	assert.True(t, found, "Collection should be created")
}
```

### 验收标准
- ✅ Qdrant 客户端成功连接
- ✅ 集合创建功能正常
- ✅ 单元测试通过
- ✅ 配置正确加载

---

## 阶段三：向量索引功能实现（第5-6天）

### 目标
实现文档向量的索引功能，支持添加、更新、删除操作。

### 任务清单

#### 3.1 向量索引接口

**文件**：`services/coordinator/internal/engine/vector_index.go`（新建）

```go
package engine

import (
	"context"
	"fmt"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/qdrant/go-client/qdrant"
)

type VectorIndex struct {
	client *QdrantClient
	logger *util.Logger
}

func NewVectorIndex(client *QdrantClient, logger *util.Logger) *VectorIndex {
	return &VectorIndex{
		client: client,
		logger: logger,
	}
}

type DocumentPoint struct {
	ID      string
	Vector  []float64
	Payload map[string]interface{}
}

func (vi *VectorIndex) Upsert(ctx context.Context, collection string, points []DocumentPoint) error {
	if len(points) == 0 {
		return nil
	}

	qdrantPoints := make([]*qdrant.PointStruct, len(points))
	for i, p := range points {
		qdrantPoints[i] = &qdrant.PointStruct{
			Id:      qdrant.NewID(p.ID),
			Vectors: qdrant.NewVectors(p.Vector),
			Payload: qdrant.NewPayloadMap(p.Payload),
		}
	}

	upsertReq := &qdrant.UpsertPoints{
		CollectionName: collection,
		Points:         qdrantPoints,
	}

	if err := vi.client.GetClient().Upsert(ctx, upsertReq); err != nil {
		return fmt.Errorf("failed to upsert points: %w", err)
	}

	vi.logger.Infof("Upserted %d points to collection %s", len(points), collection)
	return nil
}

func (vi *VectorIndex) Delete(ctx context.Context, collection string, ids []string) error {
	if len(ids) == 0 {
		return nil
	}

	qdrantIDs := make([]qdrant.ID, len(ids))
	for i, id := range ids {
		qdrantIDs[i] = qdrant.NewID(id)
	}

	deleteReq := &qdrant.DeletePoints{
		CollectionName: collection,
		Points:         qdrantIDs,
	}

	if err := vi.client.GetClient().Delete(ctx, deleteReq); err != nil {
		return fmt.Errorf("failed to delete points: %w", err)
	}

	vi.logger.Infof("Deleted %d points from collection %s", len(ids), collection)
	return nil
}

func (vi *VectorIndex) UpdatePayload(ctx context.Context, collection string, id string, payload map[string]interface{}) error {
	setPayloadReq := &qdrant.SetPayload{
		CollectionName: collection,
		Payload:        qdrant.NewPayloadMap(payload),
		PointsSelector: qdrant.NewPointsSelector(qdrant.NewID(id)),
	}

	if err := vi.client.GetClient().SetPayload(ctx, setPayloadReq); err != nil {
		return fmt.Errorf("failed to update payload: %w", err)
	}

	vi.logger.Infof("Updated payload for point %s in collection %s", id, collection)
	return nil
}

func (vi *VectorIndex) GetPoint(ctx context.Context, collection string, id string) (*DocumentPoint, error) {
	retrieveReq := &qdrant.RetrievePoints{
		CollectionName: collection,
		Ids:            []qdrant.ID{qdrant.NewID(id)},
		WithPayload:    &qdrant.WithPayloadSelector{SelectorOptions: &qdrant.WithPayloadSelector_Enable{Enable: true}},
		WithVectors:    &qdrant.WithVectors{Enable: true},
	}

	points, err := vi.client.GetClient().Retrieve(ctx, retrieveReq)
	if err != nil {
		return nil, fmt.Errorf("failed to retrieve point: %w", err)
	}

	if len(points) == 0 {
		return nil, fmt.Errorf("point not found: %s", id)
	}

	point := points[0]
	var vector []float64
	if point.Vectors != nil {
		vector = point.Vectors.GetVector().Data
	}

	payload := make(map[string]interface{})
	if point.Payload != nil {
		for k, v := range point.Payload.AsMap() {
			payload[k] = v
		}
	}

	return &DocumentPoint{
		ID:      point.Id.GetNum(),
		Vector:  vector,
		Payload: payload,
	}, nil
}
```

#### 3.2 嵌入模型服务集成

**文件**：`services/coordinator/internal/engine/embedding_service.go`（新建）

```go
package engine

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

type EmbeddingService struct {
	baseURL    string
	model      string
	dimension  int
	httpClient *http.Client
}

func NewEmbeddingService(baseURL, model string, dimension int) *EmbeddingService {
	return &EmbeddingService{
		baseURL: baseURL,
		model:   model,
		dimension: dimension,
		httpClient: &http.Client{
			Timeout: 30 * time.Second,
		},
	}
}

type EmbeddingRequest struct {
	Texts []string `json:"texts"`
	Model string   `json:"model,omitempty"`
}

type EmbeddingResponse struct {
	Embeddings [][]float64 `json:"embeddings"`
	Model     string      `json:"model"`
}

func (es *EmbeddingService) GenerateEmbedding(ctx context.Context, text string) ([]float64, error) {
	req := EmbeddingRequest{
		Texts: []string{text},
		Model: es.model,
	}

	reqBody, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, "POST", es.baseURL+"/embeddings", io.NopCloser(bytes.NewReader(reqBody)))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := es.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("embedding service returned status %d: %s", resp.StatusCode, string(body))
	}

	var embeddingResp EmbeddingResponse
	if err := json.NewDecoder(resp.Body).Decode(&embeddingResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	if len(embeddingResp.Embeddings) == 0 {
		return nil, fmt.Errorf("no embeddings returned")
	}

	return embeddingResp.Embeddings[0], nil
}

func (es *EmbeddingService) GenerateBatchEmbeddings(ctx context.Context, texts []string) ([][]float64, error) {
	req := EmbeddingRequest{
		Texts: texts,
		Model: es.model,
	}

	reqBody, err := json.Marshal(req)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal request: %w", err)
	}

	httpReq, err := http.NewRequestWithContext(ctx, "POST", es.baseURL+"/embeddings", io.NopCloser(bytes.NewReader(reqBody)))
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := es.httpClient.Do(httpReq)
	if err != nil {
		return nil, fmt.Errorf("failed to send request: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return nil, fmt.Errorf("embedding service returned status %d: %s", resp.StatusCode, string(body))
	}

	var embeddingResp EmbeddingResponse
	if err := json.NewDecoder(resp.Body).Decode(&embeddingResp); err != nil {
		return nil, fmt.Errorf("failed to decode response: %w", err)
	}

	return embeddingResp.Embeddings, nil
}

func (es *EmbeddingService) GetDimension() int {
	return es.dimension
}
```

#### 3.3 单元测试

**文件**：`services/coordinator/internal/engine/vector_index_test.go`（新建）

```go
package engine

import (
	"context"
	"testing"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestVectorIndex_Upsert(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"test-index": {
				Dimension: 128,
				Distance:  "Cosine",
				HNSWConfig: &config.HNSWConfig{
					M:                 16,
					EfConstruct:       200,
					FullScanThreshold: 10000,
				},
				OptimizersConfig: &config.OptimizersConfig{
					IndexingThreshold: 10000,
					MaxSegmentSize:    200000,
					MemmapThreshold:   20000,
				},
			},
		},
	}

	logger := util.NewLogger("info")
	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	index := NewVectorIndex(client, logger)
	ctx := context.Background()

	points := []DocumentPoint{
		{
			ID:     "test-1",
			Vector: make([]float64, 128),
			Payload: map[string]interface{}{
				"title":   "Test Document 1",
				"content": "This is a test document",
			},
		},
		{
			ID:     "test-2",
			Vector: make([]float64, 128),
			Payload: map[string]interface{}{
				"title":   "Test Document 2",
				"content": "Another test document",
			},
		},
	}

	err = index.Upsert(ctx, "test-index", points)
	require.NoError(t, err)

	point, err := index.GetPoint(ctx, "test-index", "test-1")
	require.NoError(t, err)
	assert.Equal(t, "test-1", point.ID)
	assert.Equal(t, "Test Document 1", point.Payload["title"])
}

func TestVectorIndex_Delete(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"test-delete": {
				Dimension: 64,
				Distance:  "Cosine",
				HNSWConfig: &config.HNSWConfig{
					M:                 16,
					EfConstruct:       200,
					FullScanThreshold: 10000,
				},
				OptimizersConfig: &config.OptimizersConfig{
					IndexingThreshold: 10000,
					MaxSegmentSize:    200000,
					MemmapThreshold:   20000,
				},
			},
		},
	}

	logger := util.NewLogger("info")
	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	index := NewVectorIndex(client, logger)
	ctx := context.Background()

	points := []DocumentPoint{
		{
			ID:     "delete-test-1",
			Vector: make([]float64, 64),
			Payload: map[string]interface{}{
				"title": "Delete Test",
			},
		},
	}

	err = index.Upsert(ctx, "test-delete", points)
	require.NoError(t, err)

	err = index.Delete(ctx, "test-delete", []string{"delete-test-1"})
	require.NoError(t, err)

	_, err = index.GetPoint(ctx, "test-delete", "delete-test-1")
	assert.Error(t, err)
}
```

### 验收标准
- ✅ 向量索引功能正常
- ✅ 批量操作支持
- ✅ Payload 更新功能正常
- ✅ 单元测试通过

---

## 阶段四：向量搜索功能实现（第7-8天）

### 目标
实现向量搜索功能，支持相似度搜索和过滤。

### 任务清单

#### 4.1 向量搜索实现

**文件**：`services/coordinator/internal/engine/vector_search.go`（新建）

```go
package engine

import (
	"context"
	"fmt"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/qdrant/go-client/qdrant"
)

type VectorSearch struct {
	client         *QdrantClient
	embeddingSvc   *EmbeddingService
	index          *VectorIndex
	cache          *util.Cache
	logger         *util.Logger
	circuitBreaker *CircuitBreaker
}

func NewVectorSearch(
	client *QdrantClient,
	embeddingSvc *EmbeddingService,
	cache *util.Cache,
	logger *util.Logger,
) *VectorSearch {
	return &VectorSearch{
		client:       client,
		embeddingSvc: embeddingSvc,
		index:        NewVectorIndex(client, logger),
		cache:        cache,
		logger:       logger,
		circuitBreaker: NewCircuitBreaker(&CircuitBreakerConfig{
			FailureThreshold: 5,
			SuccessThreshold: 2,
			Timeout:          30 * time.Second,
		}),
	}
}

func (vs *VectorSearch) Search(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	if !vs.circuitBreaker.AllowRequest() {
		return nil, fmt.Errorf("circuit breaker is open for vector search")
	}

	startTime := time.Now()

	cacheKey := vs.generateCacheKey(req)
	if cached, found := vs.cache.Get(cacheKey); found {
		vs.logger.Debugf("Vector search cache hit for query: %s", req.Query)
		return cached.(*model.EngineResult), nil
	}

	result, err := vs.doSearch(ctx, req)
	if err != nil {
		vs.circuitBreaker.RecordFailure()
		return nil, fmt.Errorf("vector search failed: %w", err)
	}

	vs.circuitBreaker.RecordSuccess()

	took := time.Since(startTime).Milliseconds()
	result.Took = float64(took)

	vs.cache.Set(cacheKey, result, time.Duration(vs.client.GetConfig().CacheTTL)*time.Second)

	return result, nil
}

func (vs *VectorSearch) doSearch(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	embedding, err := vs.embeddingSvc.GenerateEmbedding(ctx, req.Query)
	if err != nil {
		return nil, fmt.Errorf("failed to generate embedding: %w", err)
	}

	collection := vs.getCollectionName(req.Index)
	limit := vs.getLimit(req.Limit)

	searchReq := &qdrant.SearchPoints{
		CollectionName: collection,
		Vector:         embedding,
		Limit:          uint64(limit),
		WithPayload:    &qdrant.WithPayloadSelector{SelectorOptions: &qdrant.WithPayloadSelector_Enable{Enable: true}},
		ScoreThreshold:  vs.getScoreThreshold(),
	}

	if req.Filter != nil {
		searchReq.Filter = vs.buildFilter(req.Filter)
	}

	searchResults, err := vs.client.GetClient().Search(ctx, searchReq)
	if err != nil {
		return nil, fmt.Errorf("failed to search: %w", err)
	}

	return vs.convertResults(searchResults, req)
}

func (vs *VectorSearch) HybridSearch(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	embedding, err := vs.embeddingSvc.GenerateEmbedding(ctx, req.Query)
	if err != nil {
		return nil, fmt.Errorf("failed to generate embedding: %w", err)
	}

	collection := vs.getCollectionName(req.Index)
	limit := vs.getLimit(req.Limit)

	searchReq := &qdrant.SearchPoints{
		CollectionName: collection,
		Vector:         embedding,
		Limit:          uint64(limit),
		WithPayload:    &qdrant.WithPayloadSelector{SelectorOptions: &qdrant.WithPayloadSelector_Enable{Enable: true}},
		QueryFilter:    vs.buildHybridFilter(req),
	}

	searchResults, err := vs.client.GetClient().Search(ctx, searchReq)
	if err != nil {
		return nil, fmt.Errorf("failed to hybrid search: %w", err)
	}

	return vs.convertResults(searchResults, req)
}

func (vs *VectorSearch) convertResults(searchResults []*qdrant.ScoredPoint, req *model.SearchRequest) (*model.EngineResult, error) {
	results := make([]model.SearchResult, len(searchResults))

	for i, sr := range searchResults {
		payload := make(map[string]interface{})
		if sr.Payload != nil {
			for k, v := range sr.Payload.AsMap() {
				payload[k] = v
			}
		}

		results[i] = model.SearchResult{
			ID:           vs.getID(sr),
			Index:        req.Index,
			Score:        float64(sr.Score),
			Title:        vs.getTitle(payload),
			Content:      vs.getContent(payload),
			EngineSource: "vector",
			Rank:         int32(i + 1),
		}
	}

	return &model.EngineResult{
		Engine:  "vector",
		Results: results,
		Total:   int64(len(results)),
		Took:    0,
	}, nil
}

func (vs *VectorSearch) generateCacheKey(req *model.SearchRequest) string {
	return fmt.Sprintf("vector:%s:%s:%d", req.Index, req.Query, req.Limit)
}

func (vs *VectorSearch) getCollectionName(index string) string {
	if index == "" {
		return "default"
	}
	return index
}

func (vs *VectorSearch) getLimit(limit int32) int {
	if limit <= 0 {
		return 10
	}
	return int(limit)
}

func (vs *VectorSearch) getScoreThreshold() *float64 {
	threshold := 0.5
	return &threshold
}

func (vs *VectorSearch) buildFilter(filter *model.Filter) *qdrant.Filter {
	return nil
}

func (vs *VectorSearch) buildHybridFilter(req *model.SearchRequest) *qdrant.Filter {
	return nil
}

func (vs *VectorSearch) getID(sr *qdrant.ScoredPoint) string {
	return sr.Id.GetNum()
}

func (vs *VectorSearch) getTitle(payload map[string]interface{}) string {
	if title, ok := payload["title"].(string); ok {
		return title
	}
	return ""
}

func (vs *VectorSearch) getContent(payload map[string]interface{}) string {
	if content, ok := payload["content"].(string); ok {
		return content
	}
	return ""
}
```

#### 4.2 更新 VectorClient

**文件**：`services/coordinator/internal/engine/vector.go`（修改）

```go
package engine

import (
	"context"
	"fmt"
	"time"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
)

type VectorClient struct {
	config       *ClientConfig
	vectorConfig *config.VectorEngineConfig
	qdrantClient *QdrantClient
	vectorSearch *VectorSearch
	logger       *util.Logger
	cache        *util.Cache
}

func NewVectorClient(
	config *ClientConfig,
	vectorConfig *config.VectorEngineConfig,
	logger *util.Logger,
	cache *util.Cache,
) (*VectorClient, error) {
	if vectorConfig == nil {
		return nil, fmt.Errorf("vectorConfig cannot be nil")
	}

	qdrantClient, err := NewQdrantClient(vectorConfig, logger)
	if err != nil {
		return nil, fmt.Errorf("failed to create Qdrant client: %w", err)
	}

	embeddingSvc := NewEmbeddingService(
		"http://embedding-service:8000",
		"sentence-transformers/all-MiniLM-L6-v2",
		384,
	)

	vectorSearch := NewVectorSearch(qdrantClient, embeddingSvc, cache, logger)

	return &VectorClient{
		config:       config,
		vectorConfig: vectorConfig,
		qdrantClient: qdrantClient,
		vectorSearch: vectorSearch,
		logger:       logger,
		cache:        cache,
	}, nil
}

func (c *VectorClient) Connect(ctx context.Context) error {
	c.logger.Infof("Vector client connected to Qdrant at %s", c.vectorConfig.QdrantURL)
	return nil
}

func (c *VectorClient) Disconnect() error {
	if c.qdrantClient != nil {
		return c.qdrantClient.Close()
	}
	return nil
}

func (c *VectorClient) Search(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	return c.vectorSearch.Search(ctx, req)
}

func (c *VectorClient) HealthCheck(ctx context.Context) bool {
	if c.qdrantClient == nil {
		return false
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	collections, err := c.qdrantClient.GetClient().ListCollections(ctx)
	return err == nil && collections != nil
}

func (c *VectorClient) GetName() string {
	return "vector"
}

func (c *VectorClient) UpsertDocument(ctx context.Context, collection string, id string, text string, metadata map[string]interface{}) error {
	embedding, err := c.vectorSearch.embeddingSvc.GenerateEmbedding(ctx, text)
	if err != nil {
		return fmt.Errorf("failed to generate embedding: %w", err)
	}

	point := DocumentPoint{
		ID:     id,
		Vector: embedding,
		Payload: map[string]interface{}{
			"text":     text,
			"metadata": metadata,
		},
	}

	return c.vectorSearch.index.Upsert(ctx, collection, []DocumentPoint{point})
}

func (c *VectorClient) DeleteDocument(ctx context.Context, collection string, id string) error {
	return c.vectorSearch.index.Delete(ctx, collection, []string{id})
}
```

#### 4.3 单元测试

**文件**：`services/coordinator/internal/engine/vector_search_test.go`（新建）

```go
package engine

import (
	"context"
	"testing"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestVectorSearch_Search(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"test-search": {
				Dimension: 384,
				Distance:  "Cosine",
				HNSWConfig: &config.HNSWConfig{
					M:                 16,
					EfConstruct:       200,
					FullScanThreshold: 10000,
				},
				OptimizersConfig: &config.OptimizersConfig{
					IndexingThreshold: 10000,
					MaxSegmentSize:    200000,
					MemmapThreshold:   20000,
				},
			},
		},
	}

	logger := util.NewLogger("info")
	cache := util.NewCache(1000, 5*time.Minute)

	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	clientConfig := &ClientConfig{
		Host:       "localhost",
		Port:       50051,
		Timeout:    5 * time.Second,
		MaxRetries: 3,
	}

	vectorClient, err := NewVectorClient(clientConfig, cfg, logger, cache)
	require.NoError(t, err)
	defer vectorClient.Disconnect()

	ctx := context.Background()

	req := &model.SearchRequest{
		Query: "test query",
		Index: "test-search",
		Limit: 10,
	}

	result, err := vectorClient.Search(ctx, req)
	require.NoError(t, err)
	assert.NotNil(t, result)
	assert.Equal(t, "vector", result.Engine)
}

func TestVectorSearch_Cache(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"test-cache": {
				Dimension: 384,
				Distance:  "Cosine",
			},
		},
	}

	logger := util.NewLogger("info")
	cache := util.NewCache(1000, 5*time.Minute)

	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	clientConfig := &ClientConfig{
		Host:       "localhost",
		Port:       50051,
		Timeout:    5 * time.Second,
		MaxRetries: 3,
	}

	vectorClient, err := NewVectorClient(clientConfig, cfg, logger, cache)
	require.NoError(t, err)
	defer vectorClient.Disconnect()

	ctx := context.Background()

	req := &model.SearchRequest{
		Query: "cache test",
		Index: "test-cache",
		Limit: 10,
	}

	result1, err := vectorClient.Search(ctx, req)
	require.NoError(t, err)

	result2, err := vectorClient.Search(ctx, req)
	require.NoError(t, err)

	assert.Equal(t, result1.Engine, result2.Engine)
}
```

### 验收标准
- ✅ 向量搜索功能正常
- ✅ 缓存功能正常
- ✅ 混合搜索支持
- ✅ 单元测试通过

---

## 阶段五：集成测试和优化（第9-10天）

### 目标
进行端到端集成测试，优化性能和稳定性。

### 任务清单

#### 5.1 集成测试

**文件**：`services/coordinator/internal/engine/integration_test.go`（新建）

```go
package engine

import (
	"context"
	"testing"
	"time"

	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestVectorIntegration_FullWorkflow(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"integration-test": {
				Dimension: 384,
				Distance:  "Cosine",
				HNSWConfig: &config.HNSWConfig{
					M:                 16,
					EfConstruct:       200,
					FullScanThreshold: 10000,
				},
				OptimizersConfig: &config.OptimizersConfig{
					IndexingThreshold: 10000,
					MaxSegmentSize:    200000,
					MemmapThreshold:   20000,
				},
			},
		},
	}

	logger := util.NewLogger("info")
	cache := util.NewCache(1000, 5*time.Minute)

	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	clientConfig := &ClientConfig{
		Host:       "localhost",
		Port:       50051,
		Timeout:    5 * time.Second,
		MaxRetries: 3,
	}

	vectorClient, err := NewVectorClient(clientConfig, cfg, logger, cache)
	require.NoError(t, err)
	defer vectorClient.Disconnect()

	ctx := context.Background()

	t.Run("Upsert and Search", func(t *testing.T) {
		err := vectorClient.UpsertDocument(ctx, "integration-test", "doc-1", "This is a test document about machine learning", map[string]interface{}{
			"title": "Machine Learning",
			"author": "Test Author",
		})
		require.NoError(t, err)

		err = vectorClient.UpsertDocument(ctx, "integration-test", "doc-2", "This is a test document about deep learning", map[string]interface{}{
			"title": "Deep Learning",
			"author": "Test Author",
		})
		require.NoError(t, err)

		req := &model.SearchRequest{
			Query: "machine learning",
			Index: "integration-test",
			Limit: 10,
		}

		result, err := vectorClient.Search(ctx, req)
		require.NoError(t, err)
		assert.Greater(t, result.Total, int64(0))
	})

	t.Run("Delete and Search", func(t *testing.T) {
		err := vectorClient.DeleteDocument(ctx, "integration-test", "doc-1")
		require.NoError(t, err)

		req := &model.SearchRequest{
			Query: "machine learning",
			Index: "integration-test",
			Limit: 10,
		}

		result, err := vectorClient.Search(ctx, req)
		require.NoError(t, err)

		found := false
		for _, r := range result.Results {
			if r.ID == "doc-1" {
				found = true
				break
			}
		}
		assert.False(t, found, "Deleted document should not be found")
	})
}

func TestVectorPerformance(t *testing.T) {
	cfg := &config.VectorEngineConfig{
		Enabled:   true,
		QdrantURL: "http://localhost:6333",
		Collections: map[string]config.CollectionConfig{
			"perf-test": {
				Dimension: 384,
				Distance:  "Cosine",
			},
		},
	}

	logger := util.NewLogger("info")
	cache := util.NewCache(1000, 5*time.Minute)

	client, err := NewQdrantClient(cfg, logger)
	require.NoError(t, err)
	defer client.Close()

	clientConfig := &ClientConfig{
		Host:       "localhost",
		Port:       50051,
		Timeout:    5 * time.Second,
		MaxRetries: 3,
	}

	vectorClient, err := NewVectorClient(clientConfig, cfg, logger, cache)
	require.NoError(t, err)
	defer vectorClient.Disconnect()

	ctx := context.Background()

	numDocs := 100
	for i := 0; i < numDocs; i++ {
		err := vectorClient.UpsertDocument(ctx, "perf-test", fmt.Sprintf("doc-%d", i), fmt.Sprintf("Test document %d content", i), map[string]interface{}{
			"title": fmt.Sprintf("Document %d", i),
		})
		require.NoError(t, err)
	}

	startTime := time.Now()
	for i := 0; i < 10; i++ {
		req := &model.SearchRequest{
			Query: fmt.Sprintf("test %d", i),
			Index: "perf-test",
			Limit: 10,
		}

		_, err := vectorClient.Search(ctx, req)
		require.NoError(t, err)
	}
	elapsed := time.Since(startTime)

	avgLatency := elapsed.Milliseconds() / 10
	t.Logf("Average search latency: %dms", avgLatency)
	assert.Less(t, avgLatency, int64(100), "Average latency should be less than 100ms")
}
```

#### 5.2 性能优化

**优化项**：
1. **连接池优化**：使用连接池管理 Qdrant 连接
2. **批量操作**：支持批量索引和搜索
3. **缓存预热**：在服务启动时预热热门查询
4. **索引优化**：调整 HNSW 参数以获得最佳性能
5. **量化配置**：启用向量量化以减少内存占用

#### 5.3 监控指标

**文件**：`services/coordinator/internal/engine/vector_metrics.go`（新建）

```go
package engine

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	vectorSearchDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "vector_search_duration_seconds",
			Help:    "Vector search duration in seconds",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"collection"},
	)

	vectorSearchTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "vector_search_total",
			Help: "Total number of vector searches",
		},
		[]string{"collection", "status"},
	)

	vectorCacheHits = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "vector_cache_hits_total",
			Help: "Total number of vector search cache hits",
		},
	)

	vectorCacheMisses = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "vector_cache_misses_total",
			Help: "Total number of vector search cache misses",
		},
	)

	vectorUpsertDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "vector_upsert_duration_seconds",
			Help:    "Vector upsert duration in seconds",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"collection"},
	)

	qdrantConnectionErrors = promauto.NewCounter(
		prometheus.CounterOpts{
			Name: "qdrant_connection_errors_total",
			Help: "Total number of Qdrant connection errors",
		},
	)
)
```

### 验收标准
- ✅ 集成测试通过
- ✅ 性能满足要求（平均延迟 < 100ms）
- ✅ 监控指标正常
- ✅ 稳定性测试通过

---

## 四、总结

### 4.1 实施时间表

| 阶段 | 时间 | 主要任务 |
|------|------|---------|
| 阶段一 | 第1-2天 | Qdrant 环境准备 |
| 阶段二 | 第3-4天 | Qdrant 客户端基础封装 |
| 阶段三 | 第5-6天 | 向量索引功能实现 |
| 阶段四 | 第7-8天 | 向量搜索功能实现 |
| 阶段五 | 第9-10天 | 集成测试和优化 |

### 4.2 关键里程碑

1. **Day 2**：Qdrant 环境搭建完成，可访问
2. **Day 4**：Qdrant 客户端基础功能完成，单元测试通过
3. **Day 6**：向量索引功能完成，支持增删改
4. **Day 8**：向量搜索功能完成，支持缓存
5. **Day 10**：集成测试通过，性能达标

### 4.3 风险和缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Qdrant 性能不达标 | 高 | 提前进行性能测试，调整 HNSW 参数 |
| 嵌入模型服务不稳定 | 中 | 实现重试机制，考虑本地模型 |
| 缓存效果不明显 | 低 | 监控缓存命中率，调整 TTL |
| 集成测试失败 | 中 | 分模块测试，逐步集成 |

### 4.4 后续优化方向

1. **混合搜索优化**：实现更精细的混合搜索策略
2. **过滤功能增强**：支持更复杂的过滤条件
3. **分布式部署**：支持 Qdrant 集群部署
4. **实时索引**：实现流式索引功能
5. **A/B 测试**：支持不同搜索策略的 A/B 测试
