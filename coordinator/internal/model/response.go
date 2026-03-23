package model

import "time"

type SearchResponse struct {
	RequestID    string         `json:"request_id"`
	Results      []SearchResult `json:"results"`
	Total        int64          `json:"total"`
	Took         float64        `json:"took_ms"`
	EnginesUsed  []string       `json:"engines_used"`
	CacheHit     bool           `json:"cache_hit"`
	QueryInfo    *QueryInfo     `json:"query_info,omitempty"`
}

type SearchResult struct {
	ID           string            `json:"id"`
	Index        string            `json:"index"`
	Score        float64           `json:"score"`
	Title        string            `json:"title,omitempty"`
	Content      string            `json:"content,omitempty"`
	Highlight    map[string]string `json:"highlight,omitempty"`
	Fields       map[string]interface{} `json:"fields,omitempty"`
	EngineSource string            `json:"engine_source,omitempty"`
	Rank         int32             `json:"rank"`
}

type EngineResult struct {
	Engine    string         `json:"engine"`
	Results   []SearchResult `json:"results"`
	Total     int64         `json:"total"`
	Took      float64       `json:"took_ms"`
	Error     string        `json:"error,omitempty"`
	TimedOut  bool          `json:"timed_out,omitempty"`
}

type DocumentResponse struct {
	ID        string                 `json:"id"`
	Index     string                 `json:"index"`
	Success   bool                   `json:"success"`
	Error     string                 `json:"error,omitempty"`
	Fields    map[string]interface{} `json:"fields,omitempty"`
}

type BulkDocumentResponse struct {
	Index      string               `json:"index"`
	Success    bool                 `json:"success"`
	Total      int                  `json:"total"`
	Successful int                  `json:"successful"`
	Failed     int                  `json:"failed"`
	Results    []DocumentResponse    `json:"results,omitempty"`
	Errors     []string             `json:"errors,omitempty"`
}

type DeleteResponse struct {
	ID      string `json:"id"`
	Index   string `json:"index"`
	Success bool   `json:"success"`
	Error   string `json:"error,omitempty"`
}

type IndexResponse struct {
	Name      string   `json:"name"`
	Success   bool     `json:"success"`
	Error     string   `json:"error,omitempty"`
	Fields    []string `json:"fields,omitempty"`
}

type IndexStatsResponse struct {
	Index         string `json:"index"`
	DocumentCount int64  `json:"document_count"`
	IndexSize     int64  `json:"index_size"`
	LastUpdated   string `json:"last_updated"`
}

type HealthCheckResponse struct {
	Service    string    `json:"service"`
	Status     string    `json:"status"`
	Version    string    `json:"version,omitempty"`
	Uptime     string    `json:"uptime,omitempty"`
	Timestamp  time.Time `json:"timestamp"`
	Engines    []EngineHealth `json:"engines,omitempty"`
}

type EngineHealth struct {
	Name      string `json:"name"`
	Status    string `json:"status"`
	Address   string `json:"address,omitempty"`
	Latency   float64 `json:"latency_ms,omitempty"`
	Error     string `json:"error,omitempty"`
}

type ErrorResponse struct {
	RequestID string `json:"request_id"`
	Code      int    `json:"code"`
	Message   string `json:"message"`
	Details   string `json:"details,omitempty"`
	Timestamp time.Time `json:"timestamp"`
}

type MergerStats struct {
	Strategy    string  `json:"strategy"`
	Took        float64 `json:"took_ms"`
	ResultsMerged int    `json:"results_merged"`
	DuplicatesRemoved int `json:"duplicates_removed"`
}

type CacheStats struct {
	Hits       int64   `json:"hits"`
	Misses     int64   `json:"misses"`
	HitRate    float64 `json:"hit_rate"`
	Size       int64   `json:"size"`
	MaxSize    int64   `json:"max_size"`
}
