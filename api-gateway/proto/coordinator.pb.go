package proto

import (
	"context"

	"google.golang.org/grpc"
)

type SearchRequest struct {
	Query     string            `json:"query"`
	Indexes   []string          `json:"indexes"`
	Page      int32             `json:"page"`
	PageSize  int32             `json:"page_size"`
	Filters   map[string]string `json:"filters"`
	Fields    []string          `json:"fields"`
	Highlight bool              `json:"highlight"`
	SortBy    string            `json:"sort_by"`
	SortOrder string            `json:"sort_order"`
	Explain   bool              `json:"explain"`
}

type SearchResponse struct {
	Results    []*SearchResult `json:"results"`
	Total      int32           `json:"total"`
	Page       int32           `json:"page"`
	PageSize   int32           `json:"page_size"`
	TotalPages int32           `json:"total_pages"`
	TookMs     float64         `json:"took_ms"`
}

type SearchResult struct {
	Id         string             `json:"id"`
	Score      float64            `json:"score"`
	Fields     map[string]string  `json:"fields"`
	Highlights map[string]string  `json:"highlights"`
	Explain    map[string]float64 `json:"explain"`
}

type GetDocumentRequest struct {
	IndexId    string `json:"index_id"`
	DocumentId string `json:"document_id"`
}

type DocumentResponse struct {
	Id     string            `json:"id"`
	Fields map[string]string `json:"fields"`
	Score  float64           `json:"score"`
}

type AddDocumentRequest struct {
	IndexId string            `json:"index_id"`
	Fields  map[string]string `json:"fields"`
}

type AddDocumentResponse struct {
	Id      string `json:"id"`
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type UpdateDocumentRequest struct {
	IndexId    string            `json:"index_id"`
	DocumentId string            `json:"document_id"`
	Fields     map[string]string `json:"fields"`
}

type UpdateDocumentResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type DeleteDocumentRequest struct {
	IndexId    string `json:"index_id"`
	DocumentId string `json:"document_id"`
}

type DeleteDocumentResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type BatchDocumentsRequest struct {
	IndexId   string              `json:"index_id"`
	Documents []map[string]string `json:"documents"`
	Refresh   bool                `json:"refresh"`
}

type BatchDocumentsResponse struct {
	SuccessCount int32    `json:"success_count"`
	FailureCount int32    `json:"failure_count"`
	Errors       []string `json:"errors"`
}

type CreateIndexRequest struct {
	Name      string            `json:"name"`
	IndexType string            `json:"index_type"`
	Fields    []string          `json:"fields"`
	Options   map[string]string `json:"options"`
}

type CreateIndexResponse struct {
	Id      string `json:"id"`
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type ListIndexesRequest struct {
	Page     int32 `json:"page"`
	PageSize int32 `json:"page_size"`
}

type ListIndexesResponse struct {
	Indexes []*IndexInfo `json:"indexes"`
	Total   int32        `json:"total"`
}

type IndexInfo struct {
	Id            string `json:"id"`
	Name          string `json:"name"`
	IndexType     string `json:"index_type"`
	DocumentCount int64  `json:"document_count"`
	SizeBytes     int64  `json:"size_bytes"`
	CreatedAt     string `json:"created_at"`
	UpdatedAt     string `json:"updated_at"`
}

type GetIndexRequest struct {
	IndexId string `json:"index_id"`
}

type GetIndexResponse struct {
	Index *IndexInfo `json:"index"`
}

type DeleteIndexRequest struct {
	IndexId string `json:"index_id"`
}

type DeleteIndexResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
}

type RebuildIndexRequest struct {
	IndexId string `json:"index_id"`
	Async   bool   `json:"async"`
}

type RebuildIndexResponse struct {
	Success bool   `json:"success"`
	Message string `json:"message"`
	TaskId  string `json:"task_id"`
}

type HealthCheckRequest struct {
	Service string `json:"service"`
}

type HealthCheckResponse struct {
	Status        string            `json:"status"`
	Version       string            `json:"version"`
	UptimeSeconds int64             `json:"uptime_seconds"`
	Details       map[string]string `json:"details"`
}

type ServiceStatus struct {
	Name      string `json:"name"`
	Address   string `json:"address"`
	Status    string `json:"status"`
	LatencyMs int64  `json:"latency_ms"`
	LastCheck string `json:"last_check"`
	Message   string `json:"message"`
}

type SearchServiceClient interface {
	Search(ctx context.Context, in *SearchRequest, opts ...grpc.CallOption) (*SearchResponse, error)
}

type DocumentServiceClient interface {
	GetDocument(ctx context.Context, in *GetDocumentRequest, opts ...grpc.CallOption) (*DocumentResponse, error)
	AddDocument(ctx context.Context, in *AddDocumentRequest, opts ...grpc.CallOption) (*AddDocumentResponse, error)
	UpdateDocument(ctx context.Context, in *UpdateDocumentRequest, opts ...grpc.CallOption) (*UpdateDocumentResponse, error)
	DeleteDocument(ctx context.Context, in *DeleteDocumentRequest, opts ...grpc.CallOption) (*DeleteDocumentResponse, error)
	BatchDocuments(ctx context.Context, in *BatchDocumentsRequest, opts ...grpc.CallOption) (*BatchDocumentsResponse, error)
}

type IndexServiceClient interface {
	CreateIndex(ctx context.Context, in *CreateIndexRequest, opts ...grpc.CallOption) (*CreateIndexResponse, error)
	ListIndexes(ctx context.Context, in *ListIndexesRequest, opts ...grpc.CallOption) (*ListIndexesResponse, error)
	GetIndex(ctx context.Context, in *GetIndexRequest, opts ...grpc.CallOption) (*GetIndexResponse, error)
	DeleteIndex(ctx context.Context, in *DeleteIndexRequest, opts ...grpc.CallOption) (*DeleteIndexResponse, error)
	RebuildIndex(ctx context.Context, in *RebuildIndexRequest, opts ...grpc.CallOption) (*RebuildIndexResponse, error)
}

type HealthClient interface {
	Check(ctx context.Context, in *HealthCheckRequest, opts ...grpc.CallOption) (*HealthCheckResponse, error)
}

type searchServiceClient struct {
	cc grpc.ClientConnInterface
}

func NewSearchServiceClient(cc grpc.ClientConnInterface) SearchServiceClient {
	return &searchServiceClient{cc}
}

func (c *searchServiceClient) Search(ctx context.Context, in *SearchRequest, opts ...grpc.CallOption) (*SearchResponse, error) {
	out := new(SearchResponse)
	err := c.cc.Invoke(ctx, "/coordinator.SearchService/Search", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

type documentServiceClient struct {
	cc grpc.ClientConnInterface
}

func NewDocumentServiceClient(cc grpc.ClientConnInterface) DocumentServiceClient {
	return &documentServiceClient{cc}
}

func (c *documentServiceClient) GetDocument(ctx context.Context, in *GetDocumentRequest, opts ...grpc.CallOption) (*DocumentResponse, error) {
	out := new(DocumentResponse)
	err := c.cc.Invoke(ctx, "/coordinator.DocumentService/GetDocument", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *documentServiceClient) AddDocument(ctx context.Context, in *AddDocumentRequest, opts ...grpc.CallOption) (*AddDocumentResponse, error) {
	out := new(AddDocumentResponse)
	err := c.cc.Invoke(ctx, "/coordinator.DocumentService/AddDocument", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *documentServiceClient) UpdateDocument(ctx context.Context, in *UpdateDocumentRequest, opts ...grpc.CallOption) (*UpdateDocumentResponse, error) {
	out := new(UpdateDocumentResponse)
	err := c.cc.Invoke(ctx, "/coordinator.DocumentService/UpdateDocument", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *documentServiceClient) DeleteDocument(ctx context.Context, in *DeleteDocumentRequest, opts ...grpc.CallOption) (*DeleteDocumentResponse, error) {
	out := new(DeleteDocumentResponse)
	err := c.cc.Invoke(ctx, "/coordinator.DocumentService/DeleteDocument", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *documentServiceClient) BatchDocuments(ctx context.Context, in *BatchDocumentsRequest, opts ...grpc.CallOption) (*BatchDocumentsResponse, error) {
	out := new(BatchDocumentsResponse)
	err := c.cc.Invoke(ctx, "/coordinator.DocumentService/BatchDocuments", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

type indexServiceClient struct {
	cc grpc.ClientConnInterface
}

func NewIndexServiceClient(cc grpc.ClientConnInterface) IndexServiceClient {
	return &indexServiceClient{cc}
}

func (c *indexServiceClient) CreateIndex(ctx context.Context, in *CreateIndexRequest, opts ...grpc.CallOption) (*CreateIndexResponse, error) {
	out := new(CreateIndexResponse)
	err := c.cc.Invoke(ctx, "/coordinator.IndexService/CreateIndex", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *indexServiceClient) ListIndexes(ctx context.Context, in *ListIndexesRequest, opts ...grpc.CallOption) (*ListIndexesResponse, error) {
	out := new(ListIndexesResponse)
	err := c.cc.Invoke(ctx, "/coordinator.IndexService/ListIndexes", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *indexServiceClient) GetIndex(ctx context.Context, in *GetIndexRequest, opts ...grpc.CallOption) (*GetIndexResponse, error) {
	out := new(GetIndexResponse)
	err := c.cc.Invoke(ctx, "/coordinator.IndexService/GetIndex", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *indexServiceClient) DeleteIndex(ctx context.Context, in *DeleteIndexRequest, opts ...grpc.CallOption) (*DeleteIndexResponse, error) {
	out := new(DeleteIndexResponse)
	err := c.cc.Invoke(ctx, "/coordinator.IndexService/DeleteIndex", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *indexServiceClient) RebuildIndex(ctx context.Context, in *RebuildIndexRequest, opts ...grpc.CallOption) (*RebuildIndexResponse, error) {
	out := new(RebuildIndexResponse)
	err := c.cc.Invoke(ctx, "/coordinator.IndexService/RebuildIndex", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

type healthClient struct {
	cc grpc.ClientConnInterface
}

func NewHealthClient(cc grpc.ClientConnInterface) HealthClient {
	return &healthClient{cc}
}

func (c *healthClient) Check(ctx context.Context, in *HealthCheckRequest, opts ...grpc.CallOption) (*HealthCheckResponse, error) {
	out := new(HealthCheckResponse)
	err := c.cc.Invoke(ctx, "/coordinator.Health/Check", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

type UnimplementedSearchServiceServer struct{}

func (UnimplementedSearchServiceServer) Search(ctx context.Context, req *SearchRequest) (*SearchResponse, error) {
	return nil, nil
}

type UnimplementedDocumentServiceServer struct{}

func (UnimplementedDocumentServiceServer) GetDocument(ctx context.Context, req *GetDocumentRequest) (*DocumentResponse, error) {
	return nil, nil
}

func (UnimplementedDocumentServiceServer) AddDocument(ctx context.Context, req *AddDocumentRequest) (*AddDocumentResponse, error) {
	return nil, nil
}

func (UnimplementedDocumentServiceServer) UpdateDocument(ctx context.Context, req *UpdateDocumentRequest) (*UpdateDocumentResponse, error) {
	return nil, nil
}

func (UnimplementedDocumentServiceServer) DeleteDocument(ctx context.Context, req *DeleteDocumentRequest) (*DeleteDocumentResponse, error) {
	return nil, nil
}

func (UnimplementedDocumentServiceServer) BatchDocuments(ctx context.Context, req *BatchDocumentsRequest) (*BatchDocumentsResponse, error) {
	return nil, nil
}

type UnimplementedIndexServiceServer struct{}

func (UnimplementedIndexServiceServer) CreateIndex(ctx context.Context, req *CreateIndexRequest) (*CreateIndexResponse, error) {
	return nil, nil
}

func (UnimplementedIndexServiceServer) ListIndexes(ctx context.Context, req *ListIndexesRequest) (*ListIndexesResponse, error) {
	return nil, nil
}

func (UnimplementedIndexServiceServer) GetIndex(ctx context.Context, req *GetIndexRequest) (*GetIndexResponse, error) {
	return nil, nil
}

func (UnimplementedIndexServiceServer) DeleteIndex(ctx context.Context, req *DeleteIndexRequest) (*DeleteIndexResponse, error) {
	return nil, nil
}

func (UnimplementedIndexServiceServer) RebuildIndex(ctx context.Context, req *RebuildIndexRequest) (*RebuildIndexResponse, error) {
	return nil, nil
}

type UnimplementedHealthServer struct{}

func (UnimplementedHealthServer) Check(ctx context.Context, req *HealthCheckRequest) (*HealthCheckResponse, error) {
	return nil, nil
}
