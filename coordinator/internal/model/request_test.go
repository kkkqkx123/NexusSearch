package model

import (
	"testing"
	"time"
)

func TestSearchRequest(t *testing.T) {
	req := SearchRequest{
		Query:     "test query",
		Index:     "test_index",
		Limit:     10,
		Offset:    0,
		Highlight: true,
		RequestID: "test-123",
	}

	if req.Query != "test query" {
		t.Errorf("Expected query 'test query', got '%s'", req.Query)
	}

	if req.Index != "test_index" {
		t.Errorf("Expected index 'test_index', got '%s'", req.Index)
	}

	if req.Limit != 10 {
		t.Errorf("Expected limit 10, got %d", req.Limit)
	}
}

func TestEngineConfig(t *testing.T) {
	config := EngineConfig{
		FlexSearch: &FlexSearchConfig{
			Fuzzy:     true,
			Fuzziness:  2,
			Phrase:     false,
			Boost:      1.5,
		},
		BM25: &BM25Config{
			K1:        1.2,
			B:         0.75,
			MinLength: 3,
			MaxLength:  100,
		},
		Vector: &VectorConfig{
			Model:     "all-MiniLM-L6-v2",
			Dimension: 384,
			Threshold: 0.7,
			TopK:      10,
			Hybrid:    true,
			Alpha:     0.5,
		},
	}

	if config.FlexSearch == nil {
		t.Error("Expected FlexSearch config to be set")
	}

	if config.FlexSearch.Fuzzy != true {
		t.Errorf("Expected fuzzy true, got %v", config.FlexSearch.Fuzzy)
	}

	if config.BM25 == nil {
		t.Error("Expected BM25 config to be set")
	}

	if config.BM25.K1 != 1.2 {
		t.Errorf("Expected K1 1.2, got %f", config.BM25.K1)
	}

	if config.Vector == nil {
		t.Error("Expected Vector config to be set")
	}

	if config.Vector.Model != "all-MiniLM-L6-v2" {
		t.Errorf("Expected model 'all-MiniLM-L6-v2', got '%s'", config.Vector.Model)
	}
}

func TestQueryInfo(t *testing.T) {
	info := QueryInfo{
		Query:       "test query",
		QueryType:   "exact",
		QueryLength: 10,
		HasWildcard: false,
		HasPhrase:   false,
		HasBoolean:  false,
		HasSpecial:  false,
		Timestamp:   time.Now(),
	}

	if info.Query != "test query" {
		t.Errorf("Expected query 'test query', got '%s'", info.Query)
	}

	if info.QueryType != "exact" {
		t.Errorf("Expected query type 'exact', got '%s'", info.QueryType)
	}

	if info.QueryLength != 10 {
		t.Errorf("Expected query length 10, got %d", info.QueryLength)
	}
}

func TestDocumentRequest(t *testing.T) {
	req := DocumentRequest{
		ID:      "doc-123",
		Index:   "test_index",
		Content: "test content",
		Title:   "Test Document",
		Fields: map[string]interface{}{
			"author": "John Doe",
			"date":   "2024-01-01",
		},
		Vector: []float64{0.1, 0.2, 0.3},
	}

	if req.ID != "doc-123" {
		t.Errorf("Expected ID 'doc-123', got '%s'", req.ID)
	}

	if req.Content != "test content" {
		t.Errorf("Expected content 'test content', got '%s'", req.Content)
	}

	if len(req.Vector) != 3 {
		t.Errorf("Expected vector length 3, got %d", len(req.Vector))
	}
}

func TestBulkDocumentRequest(t *testing.T) {
	req := BulkDocumentRequest{
		Index: "test_index",
		Documents: []DocumentRequest{
			{
				ID:      "doc-1",
				Index:   "test_index",
				Content: "content 1",
			},
			{
				ID:      "doc-2",
				Index:   "test_index",
				Content: "content 2",
			},
		},
	}

	if req.Index != "test_index" {
		t.Errorf("Expected index 'test_index', got '%s'", req.Index)
	}

	if len(req.Documents) != 2 {
		t.Errorf("Expected 2 documents, got %d", len(req.Documents))
	}
}
