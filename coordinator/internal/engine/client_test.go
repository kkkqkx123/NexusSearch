package engine

import (
	"context"
	"testing"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
)

func TestCircuitBreaker(t *testing.T) {
	config := &CircuitBreakerConfig{
		FailureThreshold: 3,
		SuccessThreshold: 2,
		Timeout:          1 * time.Second,
	}

	cb := NewCircuitBreaker(config)

	if cb.GetState() != StateClosed {
		t.Errorf("Expected initial state to be Closed, got %v", cb.GetState())
	}

	if !cb.AllowRequest() {
		t.Error("Expected AllowRequest to return true in Closed state")
	}

	for i := 0; i < 3; i++ {
		cb.RecordFailure()
	}

	if cb.GetState() != StateOpen {
		t.Errorf("Expected state to be Open after failures, got %v", cb.GetState())
	}

	if cb.AllowRequest() {
		t.Error("Expected AllowRequest to return false in Open state")
	}

	time.Sleep(1100 * time.Millisecond)

	if !cb.AllowRequest() {
		t.Error("Expected AllowRequest to return true after timeout")
	}

	if cb.GetState() != StateHalfOpen {
		t.Errorf("Expected state to be HalfOpen after timeout, got %v", cb.GetState())
	}

	cb.RecordSuccess()
	cb.RecordSuccess()

	if cb.GetState() != StateClosed {
		t.Errorf("Expected state to be Closed after successes, got %v", cb.GetState())
	}
}

func TestFlexSearchClient(t *testing.T) {
	logger, err := util.NewLogger("info", "json", "stdout")
	if err != nil {
		t.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	config := &ClientConfig{
		Host:       "localhost",
		Port:       50053,
		Timeout:    5 * time.Second,
		MaxRetries: 2,
		PoolSize:   5,
	}

	client := NewFlexSearchClient(config, logger)

	if client.GetName() != "flexsearch" {
		t.Errorf("Expected name to be flexsearch, got %s", client.GetName())
	}

	ctx := context.Background()
	req := &model.SearchRequest{
		Query: "test query",
		Index: "test_index",
		Limit: 10,
	}

	result, err := client.Search(ctx, req)
	if err != nil {
		t.Errorf("Search failed: %v", err)
	}

	if result.Engine != "flexsearch" {
		t.Errorf("Expected engine to be flexsearch, got %s", result.Engine)
	}

	if len(result.Results) != 10 {
		t.Errorf("Expected 10 results, got %d", len(result.Results))
	}
}

func TestBM25Client(t *testing.T) {
	logger, err := util.NewLogger("info", "json", "stdout")
	if err != nil {
		t.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	config := &ClientConfig{
		Host:       "localhost",
		Port:       50054,
		Timeout:    5 * time.Second,
		MaxRetries: 2,
		PoolSize:   5,
	}

	bm25Config := &BM25EngineConfig{
		K1:        1.2,
		B:         0.75,
		MinLength: 2,
		MaxLength: 100,
	}

	client := NewBM25Client(config, bm25Config, logger)

	if client.GetName() != "bm25" {
		t.Errorf("Expected name to be bm25, got %s", client.GetName())
	}

	ctx := context.Background()
	req := &model.SearchRequest{
		Query: "test query for bm25",
		Index: "test_index",
		Limit: 10,
	}

	result, err := client.Search(ctx, req)
	if err != nil {
		t.Errorf("Search failed: %v", err)
	}

	if result.Engine != "bm25" {
		t.Errorf("Expected engine to be bm25, got %s", result.Engine)
	}

	if len(result.Results) != 10 {
		t.Errorf("Expected 10 results, got %d", len(result.Results))
	}
}

func TestVectorClient(t *testing.T) {
	logger, err := util.NewLogger("info", "json", "stdout")
	if err != nil {
		t.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	config := &ClientConfig{
		Host:       "localhost",
		Port:       50055,
		Timeout:    5 * time.Second,
		MaxRetries: 2,
		PoolSize:   5,
	}

	vectorConfig := &VectorEngineConfig{
		Model:     "all-MiniLM-L6-v2",
		Dimension: 384,
		Threshold: 0.7,
		TopK:      10,
		Hybrid:    false,
		Alpha:     0.5,
	}

	client, err := NewVectorClient(config, vectorConfig, logger)
	if err != nil {
		t.Fatalf("Failed to create Vector client: %v", err)
	}

	if client.GetName() != "vector" {
		t.Errorf("Expected name to be vector, got %s", client.GetName())
	}

	ctx := context.Background()
	req := &model.SearchRequest{
		Query: "test query for vector search",
		Index: "test_index",
		Limit: 10,
	}

	result, err := client.Search(ctx, req)
	if err != nil {
		t.Errorf("Search failed: %v", err)
	}

	if result.Engine != "vector" {
		t.Errorf("Expected engine to be vector, got %s", result.Engine)
	}
}
