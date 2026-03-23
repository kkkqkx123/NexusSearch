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
	Enabled    bool          `mapstructure:"enabled"`
	Host       string        `mapstructure:"host"`
	Port       int           `mapstructure:"port"`
	Timeout    time.Duration `mapstructure:"timeout"`
	MaxRetries int           `mapstructure:"max_retries"`
	PoolSize   int           `mapstructure:"pool_size"`
	Model      string        `mapstructure:"model"`
	Dimension  int           `mapstructure:"dimension"`
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
