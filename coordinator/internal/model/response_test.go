package model

import (
	"testing"
	"time"
)

func TestSearchResponse(t *testing.T) {
	resp := SearchResponse{
		RequestID:   "test-123",
		Results:     []SearchResult{},
		Total:       0,
		Took:        100.5,
		EnginesUsed: []string{"flexsearch"},
		CacheHit:    false,
	}

	if resp.RequestID != "test-123" {
		t.Errorf("Expected request ID 'test-123', got '%s'", resp.RequestID)
	}

	if resp.Total != 0 {
		t.Errorf("Expected total 0, got %d", resp.Total)
	}

	if resp.Took != 100.5 {
		t.Errorf("Expected took 100.5, got %f", resp.Took)
	}

	if len(resp.EnginesUsed) != 1 {
		t.Errorf("Expected 1 engine, got %d", len(resp.EnginesUsed))
	}
}

func TestSearchResult(t *testing.T) {
	result := SearchResult{
		ID:           "doc-123",
		Index:        "test_index",
		Score:        0.95,
		Title:        "Test Document",
		Content:      "Test content",
		EngineSource: "flexsearch",
		Rank:         1,
		Highlight: map[string]string{
			"content": "Test <em>content</em>",
		},
		Fields: map[string]interface{}{
			"author": "John Doe",
		},
	}

	if result.ID != "doc-123" {
		t.Errorf("Expected ID 'doc-123', got '%s'", result.ID)
	}

	if result.Score != 0.95 {
		t.Errorf("Expected score 0.95, got %f", result.Score)
	}

	if result.Rank != 1 {
		t.Errorf("Expected rank 1, got %d", result.Rank)
	}

	if result.EngineSource != "flexsearch" {
		t.Errorf("Expected engine source 'flexsearch', got '%s'", result.EngineSource)
	}
}

func TestEngineResult(t *testing.T) {
	result := EngineResult{
		Engine:   "flexsearch",
		Results:  []SearchResult{},
		Total:    10,
		Took:     50.5,
		Error:    "",
		TimedOut: false,
	}

	if result.Engine != "flexsearch" {
		t.Errorf("Expected engine 'flexsearch', got '%s'", result.Engine)
	}

	if result.Total != 10 {
		t.Errorf("Expected total 10, got %d", result.Total)
	}

	if result.Took != 50.5 {
		t.Errorf("Expected took 50.5, got %f", result.Took)
	}

	if result.TimedOut != false {
		t.Errorf("Expected timed out false, got %v", result.TimedOut)
	}
}

func TestDocumentResponse(t *testing.T) {
	resp := DocumentResponse{
		ID:      "doc-123",
		Index:   "test_index",
		Success: true,
		Error:   "",
		Fields: map[string]interface{}{
			"author": "John Doe",
			"title":  "Test Document",
		},
	}

	if resp.ID != "doc-123" {
		t.Errorf("Expected ID 'doc-123', got '%s'", resp.ID)
	}

	if resp.Success != true {
		t.Errorf("Expected success true, got %v", resp.Success)
	}

	if resp.Fields["author"] != "John Doe" {
		t.Errorf("Expected author 'John Doe', got '%v'", resp.Fields["author"])
	}
}

func TestBulkDocumentResponse(t *testing.T) {
	resp := BulkDocumentResponse{
		Index:      "test_index",
		Success:    true,
		Total:      10,
		Successful:  8,
		Failed:     2,
		Results:    []DocumentResponse{},
		Errors:     []string{"error 1", "error 2"},
	}

	if resp.Index != "test_index" {
		t.Errorf("Expected index 'test_index', got '%s'", resp.Index)
	}

	if resp.Total != 10 {
		t.Errorf("Expected total 10, got %d", resp.Total)
	}

	if resp.Successful != 8 {
		t.Errorf("Expected successful 8, got %d", resp.Successful)
	}

	if resp.Failed != 2 {
		t.Errorf("Expected failed 2, got %d", resp.Failed)
	}

	if len(resp.Errors) != 2 {
		t.Errorf("Expected 2 errors, got %d", len(resp.Errors))
	}
}

func TestHealthCheckResponse(t *testing.T) {
	resp := HealthCheckResponse{
		Service:   "coordinator",
		Status:    "ok",
		Version:   "1.0.0",
		Uptime:    "1h30m",
		Timestamp: time.Now(),
		Engines: []EngineHealth{
			{
				Name:    "flexsearch",
				Status:  "ok",
				Address: "localhost:50053",
				Latency: 10.5,
			},
			{
				Name:    "bm25",
				Status:  "ok",
				Address: "localhost:50054",
				Latency: 15.2,
			},
		},
	}

	if resp.Service != "coordinator" {
		t.Errorf("Expected service 'coordinator', got '%s'", resp.Service)
	}

	if resp.Status != "ok" {
		t.Errorf("Expected status 'ok', got '%s'", resp.Status)
	}

	if len(resp.Engines) != 2 {
		t.Errorf("Expected 2 engines, got %d", len(resp.Engines))
	}

	if resp.Engines[0].Name != "flexsearch" {
		t.Errorf("Expected first engine 'flexsearch', got '%s'", resp.Engines[0].Name)
	}
}

func TestEngineHealth(t *testing.T) {
	health := EngineHealth{
		Name:    "flexsearch",
		Status:  "ok",
		Address: "localhost:50053",
		Latency: 10.5,
		Error:   "",
	}

	if health.Name != "flexsearch" {
		t.Errorf("Expected name 'flexsearch', got '%s'", health.Name)
	}

	if health.Status != "ok" {
		t.Errorf("Expected status 'ok', got '%s'", health.Status)
	}

	if health.Latency != 10.5 {
		t.Errorf("Expected latency 10.5, got %f", health.Latency)
	}
}

func TestErrorResponse(t *testing.T) {
	err := ErrorResponse{
		RequestID: "test-123",
		Code:      500,
		Message:   "Internal server error",
		Details:   "Database connection failed",
		Timestamp: time.Now(),
	}

	if err.RequestID != "test-123" {
		t.Errorf("Expected request ID 'test-123', got '%s'", err.RequestID)
	}

	if err.Code != 500 {
		t.Errorf("Expected code 500, got %d", err.Code)
	}

	if err.Message != "Internal server error" {
		t.Errorf("Expected message 'Internal server error', got '%s'", err.Message)
	}
}

func TestMergerStats(t *testing.T) {
	stats := MergerStats{
		Strategy:          "rrf",
		Took:              25.5,
		ResultsMerged:     100,
		DuplicatesRemoved:  15,
	}

	if stats.Strategy != "rrf" {
		t.Errorf("Expected strategy 'rrf', got '%s'", stats.Strategy)
	}

	if stats.Took != 25.5 {
		t.Errorf("Expected took 25.5, got %f", stats.Took)
	}

	if stats.ResultsMerged != 100 {
		t.Errorf("Expected results merged 100, got %d", stats.ResultsMerged)
	}

	if stats.DuplicatesRemoved != 15 {
		t.Errorf("Expected duplicates removed 15, got %d", stats.DuplicatesRemoved)
	}
}

func TestCacheStats(t *testing.T) {
	stats := CacheStats{
		Hits:    1000,
		Misses:  200,
		HitRate: 0.8333,
		Size:     5000,
		MaxSize:  10000,
	}

	if stats.Hits != 1000 {
		t.Errorf("Expected hits 1000, got %d", stats.Hits)
	}

	if stats.Misses != 200 {
		t.Errorf("Expected misses 200, got %d", stats.Misses)
	}

	if stats.HitRate < 0.83 || stats.HitRate > 0.84 {
		t.Errorf("Expected hit rate around 0.8333, got %f", stats.HitRate)
	}

	if stats.Size != 5000 {
		t.Errorf("Expected size 5000, got %d", stats.Size)
	}

	if stats.MaxSize != 10000 {
		t.Errorf("Expected max size 10000, got %d", stats.MaxSize)
	}
}
