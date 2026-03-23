package client

import (
	"context"
	"time"

	"github.com/flexsearch/api-gateway/internal/config"
	"github.com/flexsearch/api-gateway/internal/util"
	pb "github.com/flexsearch/api-gateway/proto"
	"google.golang.org/grpc"
)

// CircuitBreakerCoordinatorClient wraps CoordinatorClient with circuit breaker pattern
type CircuitBreakerCoordinatorClient struct {
	*CoordinatorClient
	searchCircuitBreaker   *util.CircuitBreaker
	documentCircuitBreaker *util.CircuitBreaker
	indexCircuitBreaker    *util.CircuitBreaker
	healthCircuitBreaker   *util.CircuitBreaker
}

// NewCircuitBreakerCoordinatorClient creates a new circuit breaker wrapped client
func NewCircuitBreakerCoordinatorClient(cfg *config.CoordinatorConfig) (*CircuitBreakerCoordinatorClient, error) {
	baseClient, err := NewCoordinatorClient(cfg)
	if err != nil {
		return nil, err
	}

	// Create circuit breakers with different configurations for different services
	searchConfig := util.DefaultCircuitBreakerConfig()
	searchConfig.FailureThreshold = 3
	searchConfig.Timeout = 10 * time.Second

	documentConfig := util.DefaultCircuitBreakerConfig()
	documentConfig.FailureThreshold = 5
	documentConfig.Timeout = 15 * time.Second

	indexConfig := util.DefaultCircuitBreakerConfig()
	indexConfig.FailureThreshold = 3
	indexConfig.Timeout = 20 * time.Second

	healthConfig := util.DefaultCircuitBreakerConfig()
	healthConfig.FailureThreshold = 2
	healthConfig.Timeout = 5 * time.Second

	return &CircuitBreakerCoordinatorClient{
		CoordinatorClient:      baseClient,
		searchCircuitBreaker:   util.NewCircuitBreaker("search-service", searchConfig),
		documentCircuitBreaker: util.NewCircuitBreaker("document-service", documentConfig),
		indexCircuitBreaker:    util.NewCircuitBreaker("index-service", indexConfig),
		healthCircuitBreaker:   util.NewCircuitBreaker("health-service", healthConfig),
	}, nil
}

// Search with circuit breaker
func (c *CircuitBreakerCoordinatorClient) Search(ctx context.Context, req *pb.SearchRequest, opts ...grpc.CallOption) (*pb.SearchResponse, error) {
	var resp *pb.SearchResponse
	var err error

	cbErr := c.searchCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.Search(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// GetDocument with circuit breaker
func (c *CircuitBreakerCoordinatorClient) GetDocument(ctx context.Context, req *pb.GetDocumentRequest, opts ...grpc.CallOption) (*pb.DocumentResponse, error) {
	var resp *pb.DocumentResponse
	var err error

	cbErr := c.documentCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.GetDocument(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// AddDocument with circuit breaker
func (c *CircuitBreakerCoordinatorClient) AddDocument(ctx context.Context, req *pb.AddDocumentRequest, opts ...grpc.CallOption) (*pb.AddDocumentResponse, error) {
	var resp *pb.AddDocumentResponse
	var err error

	cbErr := c.documentCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.AddDocument(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// UpdateDocument with circuit breaker
func (c *CircuitBreakerCoordinatorClient) UpdateDocument(ctx context.Context, req *pb.UpdateDocumentRequest, opts ...grpc.CallOption) (*pb.UpdateDocumentResponse, error) {
	var resp *pb.UpdateDocumentResponse
	var err error

	cbErr := c.documentCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.UpdateDocument(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// DeleteDocument with circuit breaker
func (c *CircuitBreakerCoordinatorClient) DeleteDocument(ctx context.Context, req *pb.DeleteDocumentRequest, opts ...grpc.CallOption) (*pb.DeleteDocumentResponse, error) {
	var resp *pb.DeleteDocumentResponse
	var err error

	cbErr := c.documentCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.DeleteDocument(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// BatchDocuments with circuit breaker
func (c *CircuitBreakerCoordinatorClient) BatchDocuments(ctx context.Context, req *pb.BatchDocumentsRequest, opts ...grpc.CallOption) (*pb.BatchDocumentsResponse, error) {
	var resp *pb.BatchDocumentsResponse
	var err error

	cbErr := c.documentCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.BatchDocuments(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// CreateIndex with circuit breaker
func (c *CircuitBreakerCoordinatorClient) CreateIndex(ctx context.Context, req *pb.CreateIndexRequest, opts ...grpc.CallOption) (*pb.CreateIndexResponse, error) {
	var resp *pb.CreateIndexResponse
	var err error

	cbErr := c.indexCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.CreateIndex(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// ListIndexes with circuit breaker
func (c *CircuitBreakerCoordinatorClient) ListIndexes(ctx context.Context, req *pb.ListIndexesRequest, opts ...grpc.CallOption) (*pb.ListIndexesResponse, error) {
	var resp *pb.ListIndexesResponse
	var err error

	cbErr := c.indexCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.ListIndexes(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// GetIndex with circuit breaker
func (c *CircuitBreakerCoordinatorClient) GetIndex(ctx context.Context, req *pb.GetIndexRequest, opts ...grpc.CallOption) (*pb.GetIndexResponse, error) {
	var resp *pb.GetIndexResponse
	var err error

	cbErr := c.indexCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.GetIndex(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// DeleteIndex with circuit breaker
func (c *CircuitBreakerCoordinatorClient) DeleteIndex(ctx context.Context, req *pb.DeleteIndexRequest, opts ...grpc.CallOption) (*pb.DeleteIndexResponse, error) {
	var resp *pb.DeleteIndexResponse
	var err error

	cbErr := c.indexCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.DeleteIndex(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// RebuildIndex with circuit breaker
func (c *CircuitBreakerCoordinatorClient) RebuildIndex(ctx context.Context, req *pb.RebuildIndexRequest, opts ...grpc.CallOption) (*pb.RebuildIndexResponse, error) {
	var resp *pb.RebuildIndexResponse
	var err error

	cbErr := c.indexCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.RebuildIndex(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// HealthCheck with circuit breaker
func (c *CircuitBreakerCoordinatorClient) HealthCheck(ctx context.Context, req *pb.HealthCheckRequest, opts ...grpc.CallOption) (*pb.HealthCheckResponse, error) {
	var resp *pb.HealthCheckResponse
	var err error

	cbErr := c.healthCircuitBreaker.Execute(ctx, func() error {
		resp, err = c.CoordinatorClient.HealthCheck(ctx, req, opts...)
		return err
	})

	if cbErr != nil {
		return nil, cbErr
	}

	return resp, err
}

// GetCircuitBreakerStats returns statistics for all circuit breakers
func (c *CircuitBreakerCoordinatorClient) GetCircuitBreakerStats() map[string]interface{} {
	return map[string]interface{}{
		"search":   c.searchCircuitBreaker.GetStats(),
		"document": c.documentCircuitBreaker.GetStats(),
		"index":    c.indexCircuitBreaker.GetStats(),
		"health":   c.healthCircuitBreaker.GetStats(),
	}
}

// Close closes the underlying connection
func (c *CircuitBreakerCoordinatorClient) Close() error {
	return c.CoordinatorClient.Close()
}
