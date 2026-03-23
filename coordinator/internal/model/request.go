package model

import "time"

type SearchRequest struct {
	Query          string            `json:"query"`
	Index          string            `json:"index"`
	Limit          int32             `json:"limit,omitempty"`
	Offset         int32             `json:"offset,omitempty"`
	Engines        []string          `json:"engines,omitempty"`
	EngineConfig   *EngineConfig     `json:"engine_config,omitempty"`
	Filters        map[string]string `json:"filters,omitempty"`
	SortBy         string            `json:"sort_by,omitempty"`
	SortOrder      string            `json:"sort_order,omitempty"`
	Highlight      bool              `json:"highlight,omitempty"`
	HighlightField string            `json:"highlight_field,omitempty"`
	Timeout        time.Duration     `json:"timeout,omitempty"`
	RequestID      string            `json:"request_id,omitempty"`
}

type EngineConfig struct {
	FlexSearch *FlexSearchConfig `json:"flexsearch,omitempty"`
	BM25       *BM25Config       `json:"bm25,omitempty"`
	Vector     *VectorConfig     `json:"vector,omitempty"`
}

type FlexSearchConfig struct {
	Fuzzy       bool    `json:"fuzzy,omitempty"`
	Fuzziness   int     `json:"fuzziness,omitempty"`
	Phrase      bool    `json:"phrase,omitempty"`
	Proximity   int     `json:"proximity,omitempty"`
	Boost       float64 `json:"boost,omitempty"`
}

type BM25Config struct {
	K1         float64 `json:"k1,omitempty"`
	B          float64 `json:"b,omitempty"`
	MinLength  int     `json:"min_length,omitempty"`
	MaxLength  int     `json:"max_length,omitempty"`
}

type VectorConfig struct {
	Model      string  `json:"model,omitempty"`
	Dimension  int     `json:"dimension,omitempty"`
	Threshold  float64 `json:"threshold,omitempty"`
	TopK       int     `json:"top_k,omitempty"`
	Hybrid     bool    `json:"hybrid,omitempty"`
	Alpha      float64 `json:"alpha,omitempty"`
}

type QueryInfo struct {
	Query         string    `json:"query"`
	QueryType     string    `json:"query_type"`
	QueryLength   int       `json:"query_length"`
	HasWildcard   bool      `json:"has_wildcard"`
	HasPhrase    bool      `json:"has_phrase"`
	HasBoolean    bool      `json:"has_boolean"`
	HasSpecial    bool      `json:"has_special"`
	Timestamp     time.Time `json:"timestamp"`
}

type DocumentRequest struct {
	ID       string                 `json:"id"`
	Index    string                 `json:"index"`
	Content  string                 `json:"content"`
	Title    string                 `json:"title,omitempty"`
	Fields   map[string]interface{} `json:"fields,omitempty"`
	Vector   []float64              `json:"vector,omitempty"`
}

type BulkDocumentRequest struct {
	Index      string         `json:"index"`
	Documents  []DocumentRequest `json:"documents"`
}

type DeleteRequest struct {
	ID      string `json:"id"`
	Index   string `json:"index"`
}

type IndexRequest struct {
	Name   string            `json:"name"`
	Fields map[string]string `json:"fields"`
}

type IndexStatsRequest struct {
	Index string `json:"index"`
}

type HealthCheckRequest struct {
	Service string `json:"service,omitempty"`
}
