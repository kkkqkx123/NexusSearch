package router

import (
	"context"
	"testing"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
)

func TestRouter_Route(t *testing.T) {
	logger, err := util.NewLogger("info", "json", "stdout")
	if err != nil {
		t.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	router := NewRouter(logger)

	tests := []struct {
		name            string
		query           string
		expectedType    string
		expectedEngines []string
	}{
		{
			name:            "Single term query",
			query:           "test",
			expectedType:    "single_term",
			expectedEngines: []string{"bm25"},
		},
		{
			name:            "Short phrase",
			query:           "test query",
			expectedType:    "short_phrase",
			expectedEngines: []string{"bm25"},
		},
		{
			name:            "Long query",
			query:           "this is a long query with many words",
			expectedType:    "long_query",
			expectedEngines: []string{"vector"},
		},
		{
			name:            "Query with quotes",
			query:           "\"exact phrase\"",
			expectedType:    "short_phrase",
			expectedEngines: []string{"bm25"},
		},
		{
			name:            "Query with wildcards",
			query:           "test*",
			expectedType:    "single_term",
			expectedEngines: []string{"bm25"},
		},
		{
			name:            "Hybrid query",
			query:           "test query with several words",
			expectedType:    "medium_phrase",
			expectedEngines: []string{"bm25", "vector"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			req := &model.SearchRequest{
				Query: tt.query,
				Index: "test_index",
				Limit: 10,
			}

			decision := router.Route(ctx, req)

			if decision.QueryInfo.QueryType != tt.expectedType {
				t.Errorf("Expected query type %s, got %s", tt.expectedType, decision.QueryInfo.QueryType)
			}

			if len(decision.Engines) == 0 {
				t.Error("Expected at least one engine")
			}
		})
	}
}

func TestOptimizer_Optimize(t *testing.T) {
	logger, err := util.NewLogger("info", "json", "stdout")
	if err != nil {
		t.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	optimizer := NewOptimizer(logger)

	tests := []struct {
		name          string
		query         string
		expectRewrite bool
	}{
		{
			name:          "Query with stop words",
			query:         "the quick brown fox",
			expectRewrite: true,
		},
		{
			name:          "Query without stop words",
			query:         "quick brown fox",
			expectRewrite: false,
		},
		{
			name:          "Query with synonyms",
			query:         "search for data",
			expectRewrite: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			req := &model.SearchRequest{
				Query: tt.query,
				Index: "test_index",
				Limit: 10,
			}

			optimized := optimizer.Optimize(ctx, req)

			if optimized.Rewritten != tt.expectRewrite {
				t.Errorf("Expected rewritten=%v, got %v", tt.expectRewrite, optimized.Rewritten)
			}

			if optimized.OriginalQuery != tt.query {
				t.Errorf("Original query mismatch")
			}
		})
	}
}

func TestLevenshteinDistance(t *testing.T) {
	tests := []struct {
		s1       string
		s2       string
		expected int
	}{
		{"kitten", "sitting", 3},
		{"test", "test", 0},
		{"test", "testing", 3},
		{"hello", "world", 4},
	}

	for _, tt := range tests {
		t.Run(tt.s1+" vs "+tt.s2, func(t *testing.T) {
			distance := levenshteinDistance(tt.s1, tt.s2)
			if distance != tt.expected {
				t.Errorf("Expected distance %d, got %d", tt.expected, distance)
			}
		})
	}
}
