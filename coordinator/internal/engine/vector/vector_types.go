package vector

type DistanceType string

const (
	DistanceCosine     DistanceType = "Cosine"
	DistanceEuclidean  DistanceType = "Euclidean"
	DistanceDotProduct DistanceType = "DotProduct"
)

type CollectionStatus string

const (
	CollectionStatusGreen  CollectionStatus = "Green"
	CollectionStatusYellow CollectionStatus = "Yellow"
	CollectionStatusRed    CollectionStatus = "Red"
	CollectionStatusGrey   CollectionStatus = "Grey"
)

type PayloadSchemaType string

const (
	PayloadSchemaKeyword  PayloadSchemaType = "keyword"
	PayloadSchemaInteger  PayloadSchemaType = "integer"
	PayloadSchemaFloat    PayloadSchemaType = "float"
	PayloadSchemaText     PayloadSchemaType = "text"
	PayloadSchemaBool     PayloadSchemaType = "bool"
	PayloadSchemaGeo      PayloadSchemaType = "geo"
	PayloadSchemaDatetime PayloadSchemaType = "datetime"
)

type HnswConfig struct {
	M                  int  `mapstructure:"m"`
	EfConstruct        int  `mapstructure:"ef_construct"`
	FullScanThreshold  int  `mapstructure:"full_scan_threshold"`
	MaxIndexingThreads int  `mapstructure:"max_indexing_threads"`
	OnDisk             bool `mapstructure:"on_disk"`
	PayloadM           int  `mapstructure:"payload_m"`
}

type QuantizationType string

const (
	QuantizationScalar  QuantizationType = "scalar"
	QuantizationProduct QuantizationType = "product"
	QuantizationBinary  QuantizationType = "binary"
)

type QuantizationConfig struct {
	Enabled    bool            `mapstructure:"enabled"`
	Type       QuantizationType `mapstructure:"type"`
	Quantile   float64         `mapstructure:"quantile"`
	AlwaysRAM  bool            `mapstructure:"always_ram"`
	Compression string         `mapstructure:"compression"`
}

type CollectionUpdateParams struct {
	VectorsConfig *VectorsConfigDiff `mapstructure:"vectors_config"`
}

type VectorsConfigDiff struct {
	OnDiskPayload *bool `mapstructure:"on_disk_payload"`
}

type CollectionConfig struct {
	Name                    string             `mapstructure:"name"`
	VectorSize              int                `mapstructure:"vector_size"`
	Distance                DistanceType       `mapstructure:"distance"`
	HnswConfig              *HnswConfig        `mapstructure:"hnsw_config"`
	QuantizationConfig      *QuantizationConfig `mapstructure:"quantization_config"`
	ReplicationFactor       int                `mapstructure:"replication_factor"`
	WriteConsistencyFactor  int                `mapstructure:"write_consistency_factor"`
	OnDiskPayload           bool               `mapstructure:"on_disk_payload"`
	ShardNumber             int                `mapstructure:"shard_number"`
}

type CollectionInfo struct {
	Name               string
	VectorCount        uint64
	IndexedVectorCount uint64
	PointsCount        uint64
	SegmentsCount      uint64
	Config             CollectionConfig
	Status             CollectionStatus
}

type VectorPoint struct {
	ID      string
	Vector  []float32
	Payload map[string]interface{}
}

type SearchQuery struct {
	Vector         []float32
	Limit          int
	Offset         int
	ScoreThreshold float32
	Filter         *VectorFilter
	WithPayload    bool
	WithVector     bool
}

func NewSearchQuery(vector []float32, limit int) *SearchQuery {
	return &SearchQuery{
		Vector:      vector,
		Limit:       limit,
		WithPayload: true,
		WithVector:  false,
	}
}

func (q *SearchQuery) SetOffset(offset int) *SearchQuery {
	q.Offset = offset
	return q
}

func (q *SearchQuery) SetScoreThreshold(threshold float32) *SearchQuery {
	q.ScoreThreshold = threshold
	return q
}

func (q *SearchQuery) SetFilter(filter *VectorFilter) *SearchQuery {
	q.Filter = filter
	return q
}

func (q *SearchQuery) SetWithPayload(withPayload bool) *SearchQuery {
	q.WithPayload = withPayload
	return q
}

func (q *SearchQuery) SetWithVector(withVector bool) *SearchQuery {
	q.WithVector = withVector
	return q
}

type SearchResult struct {
	ID      string
	Score   float32
	Payload map[string]interface{}
	Vector  []float32
}

type UpsertResult struct {
	OperationID uint64
	Status      UpsertStatus
}

type UpsertStatus string

const (
	UpsertStatusCompleted   UpsertStatus = "completed"
	UpsertStatusAcknowledged UpsertStatus = "acknowledged"
)

type DeleteResult struct {
	OperationID  uint64
	DeletedCount uint64
}

type HealthStatus struct {
	IsHealthy     bool
	EngineName    string
	EngineVersion string
	Message       string
}

type ScrollOptions struct {
	Limit       int
	Offset      string
	Filter      *VectorFilter
	WithPayload bool
	WithVector  bool
}

type ScrollResult struct {
	Points     []*VectorPoint
	NextOffset string
}

type PayloadIndexInfo struct {
	Field string
	Type  PayloadSchemaType
}
