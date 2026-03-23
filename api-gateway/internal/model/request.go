package model

type SearchRequest struct {
	Query     string            `json:"query" binding:"required,min=1,max=100"`
	Indexes   []string          `json:"indexes"`
	Page      int               `json:"page" binding:"omitempty,min=1"`
	PageSize  int               `json:"page_size" binding:"omitempty,min=1,max=100"`
	Filters   map[string]string `json:"filters"`
	Fields    []string          `json:"fields"`
	Highlight bool              `json:"highlight"`
	SortBy    string            `json:"sort_by"`
	SortOrder string            `json:"sort_order"`
	Explain   bool              `json:"explain"`
}

type SearchResponse struct {
	Results    []SearchResult `json:"results"`
	Total      int            `json:"total"`
	Page       int            `json:"page"`
	PageSize   int            `json:"page_size"`
	TotalPages int            `json:"total_pages"`
	TookMs     float64        `json:"took_ms"`
}

type SearchResult struct {
	ID         string            `json:"id"`
	Score      float64           `json:"score"`
	Fields     map[string]string `json:"fields"`
	Highlights map[string]string `json:"highlights,omitempty"`
}

type AddDocumentRequest struct {
	IndexID string            `json:"index_id" binding:"required"`
	Fields  map[string]string `json:"fields" binding:"required"`
}

type AddDocumentResponse struct {
	ID      string `json:"id"`
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
}

type GetDocumentRequest struct {
	IndexID    string `json:"index_id"`
	DocumentID string `json:"document_id"`
}

type DocumentResponse struct {
	ID     string            `json:"id"`
	Fields map[string]string `json:"fields"`
	Score  float64           `json:"score,omitempty"`
}

type UpdateDocumentRequest struct {
	IndexID string            `json:"index_id" binding:"required"`
	Fields  map[string]string `json:"fields" binding:"required"`
}

type UpdateDocumentResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
}

type DeleteDocumentRequest struct {
	IndexID    string `json:"index_id"`
	DocumentID string `json:"document_id"`
}

type DeleteDocumentResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
}

type BatchDocumentsRequest struct {
	IndexID   string              `json:"index_id" binding:"required"`
	Documents []map[string]string `json:"documents" binding:"required,min=1,max=100"`
	Refresh   bool                `json:"refresh"`
}

type BatchDocumentsResponse struct {
	SuccessCount int      `json:"success_count"`
	FailureCount int      `json:"failure_count"`
	Errors       []string `json:"errors,omitempty"`
}

type CreateIndexRequest struct {
	Name      string            `json:"name" binding:"required,min=1,max=100"`
	IndexType string            `json:"index_type" binding:"required"`
	Fields    []string          `json:"fields"`
	Options   map[string]string `json:"options"`
}

type CreateIndexResponse struct {
	ID      string `json:"id"`
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
}

type ListIndexesRequest struct {
	Page     int `json:"page"`
	PageSize int `json:"page_size"`
}

type ListIndexesResponse struct {
	Indexes []IndexInfo `json:"indexes"`
	Total   int         `json:"total"`
}

type IndexInfo struct {
	ID            string `json:"id"`
	Name          string `json:"name"`
	IndexType     string `json:"index_type"`
	DocumentCount int    `json:"document_count"`
	SizeBytes     int    `json:"size_bytes"`
	CreatedAt     string `json:"created_at,omitempty"`
	UpdatedAt     string `json:"updated_at,omitempty"`
}

type GetIndexRequest struct {
	IndexID string `json:"index_id"`
}

type GetIndexResponse struct {
	Index IndexInfo `json:"index"`
}

type DeleteIndexRequest struct {
	IndexID string `json:"index_id"`
}

type DeleteIndexResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
}

type RebuildIndexRequest struct {
	IndexID string `json:"index_id"`
	Async   bool   `json:"async"`
}

type RebuildIndexResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message,omitempty"`
	TaskID  string `json:"task_id,omitempty"`
}

type ErrorResponse struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	Details string `json:"details,omitempty"`
}

type SuccessResponse struct {
	Message string      `json:"message"`
	Data    interface{} `json:"data,omitempty"`
}
