package config

import (
	"strconv"
	"time"
)

type EnginesConfig struct {
	FlexSearch FlexSearchConfig `mapstructure:"flexsearch"`
	BM25       BM25Config       `mapstructure:"bm25"`
	Vector     VectorConfig     `mapstructure:"vector"`
}

type FlexSearchConfig struct {
	Enabled    bool          `mapstructure:"enabled"`
	Host       string        `mapstructure:"host"`
	Port       int           `mapstructure:"port"`
	Timeout    time.Duration `mapstructure:"timeout"`
	MaxRetries int           `mapstructure:"max_retries"`
	PoolSize   int           `mapstructure:"pool_size"`
}

type BM25Config struct {
	Enabled    bool          `mapstructure:"enabled"`
	Host       string        `mapstructure:"host"`
	Port       int           `mapstructure:"port"`
	Timeout    time.Duration `mapstructure:"timeout"`
	MaxRetries int           `mapstructure:"max_retries"`
	PoolSize   int           `mapstructure:"pool_size"`
	K1         float64       `mapstructure:"k1"`
	B          float64       `mapstructure:"b"`
}

type VectorConfig struct {
	Enabled    bool                    `mapstructure:"enabled"`
	Host       string                  `mapstructure:"host"`
	Port       int                     `mapstructure:"port"`
	Timeout    time.Duration           `mapstructure:"timeout"`
	MaxRetries int                     `mapstructure:"max_retries"`
	PoolSize   int                     `mapstructure:"pool_size"`
	Model      string                  `mapstructure:"model"`
	Dimension  int                     `mapstructure:"dimension"`
	QdrantURL  string                  `mapstructure:"qdrant_url"`
	GRPCURL    string                  `mapstructure:"grpc_url"`
	Collections map[string]CollectionConfig `mapstructure:"collections"`
	CacheTTL   int                     `mapstructure:"cache_ttl"`
}

type CollectionConfig struct {
	Dimension          int                    `mapstructure:"dimension"`
	Distance           string                 `mapstructure:"distance"`
	HNSWConfig         *HNSWConfig            `mapstructure:"hnsw_config"`
	OptimizersConfig   *OptimizersConfig      `mapstructure:"optimizers_config"`
	QuantizationConfig *QuantizationConfig    `mapstructure:"quantization_config"`
}

type HNSWConfig struct {
	M                 int `mapstructure:"m"`
	EfConstruct       int `mapstructure:"ef_construct"`
	FullScanThreshold int `mapstructure:"full_scan_threshold"`
}

type OptimizersConfig struct {
	IndexingThreshold int `mapstructure:"indexing_threshold"`
	MaxSegmentSize    int `mapstructure:"max_segment_size"`
	MemmapThreshold   int `mapstructure:"memmap_threshold"`
}

type QuantizationConfig struct {
	Scalar *ScalarQuantization `mapstructure:"scalar"`
}

type ScalarQuantization struct {
	Type      string `mapstructure:"type"`
	AlwaysRAM bool   `mapstructure:"always_ram"`
}

type VectorEngineConfig struct {
	Enabled    bool                    `mapstructure:"enabled"`
	QdrantURL  string                  `mapstructure:"qdrant_url"`
	GRPCURL    string                  `mapstructure:"grpc_url"`
	Collections map[string]CollectionConfig `mapstructure:"collections"`
	Timeout    time.Duration           `mapstructure:"timeout"`
	MaxRetries int                     `mapstructure:"max_retries"`
	CacheTTL   int                     `mapstructure:"cache_ttl"`
}

func (e *EnginesConfig) GetFlexSearchAddress() string {
	return e.FlexSearch.Address()
}

func (e *EnginesConfig) GetBM25Address() string {
	return e.BM25.Address()
}

func (e *EnginesConfig) GetVectorAddress() string {
	return e.Vector.Address()
}

func (f *FlexSearchConfig) Address() string {
	return f.Host + ":" + strconv.Itoa(f.Port)
}

func (b *BM25Config) Address() string {
	return b.Host + ":" + strconv.Itoa(b.Port)
}

func (v *VectorConfig) Address() string {
	return v.Host + ":" + strconv.Itoa(v.Port)
}
