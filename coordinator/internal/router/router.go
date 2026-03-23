package router

import (
	"context"
	"regexp"
	"strings"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
)

type Router struct {
	logger  *util.Logger
	strategies map[string]RoutingStrategy
}

type RoutingStrategy interface {
	Name() string
	ShouldRoute(ctx context.Context, req *model.SearchRequest) bool
	GetEngines() []string
	GetWeights() map[string]float64
}

type ExactMatchStrategy struct{}

func (s *ExactMatchStrategy) Name() string {
	return "exact_match"
}

func (s *ExactMatchStrategy) ShouldRoute(ctx context.Context, req *model.SearchRequest) bool {
	query := strings.TrimSpace(req.Query)
	
	words := strings.Fields(query)
	if len(words) == 0 {
		return false
	}
	
	if len(words) <= 3 {
		return true
	}
	
	hasQuotes := strings.Contains(query, "\"")
	hasWildcards := strings.ContainsAny(query, "*?")
	
	return hasQuotes || hasWildcards || len(query) <= 20
}

func (s *ExactMatchStrategy) GetEngines() []string {
	return []string{"bm25"}
}

func (s *ExactMatchStrategy) GetWeights() map[string]float64 {
	return map[string]float64{
		"bm25": 1.0,
	}
}

type FuzzySearchStrategy struct{}

func (s *FuzzySearchStrategy) Name() string {
	return "fuzzy_search"
}

func (s *FuzzySearchStrategy) ShouldRoute(ctx context.Context, req *model.SearchRequest) bool {
	query := strings.TrimSpace(req.Query)
	
	if req.EngineConfig != nil && req.EngineConfig.FlexSearch != nil {
		if req.EngineConfig.FlexSearch.Fuzzy {
			return true
		}
	}
	
	hasTypos := detectPotentialTypos(query)
	hasWildcards := strings.ContainsAny(query, "*?")
	
	return hasTypos || hasWildcards
}

func (s *FuzzySearchStrategy) GetEngines() []string {
	return []string{"flexsearch"}
}

func (s *FuzzySearchStrategy) GetWeights() map[string]float64 {
	return map[string]float64{
		"flexsearch": 1.0,
	}
}

type SemanticSearchStrategy struct{}

func (s *SemanticSearchStrategy) Name() string {
	return "semantic_search"
}

func (s *SemanticSearchStrategy) ShouldRoute(ctx context.Context, req *model.SearchRequest) bool {
	query := strings.TrimSpace(req.Query)
	
	words := strings.Fields(query)
	
	if len(words) >= 4 {
		return true
	}
	
	hasStopWords := containsStopWords(query)
	
	return len(words) >= 3 && hasStopWords
}

func (s *SemanticSearchStrategy) GetEngines() []string {
	return []string{"vector"}
}

func (s *SemanticSearchStrategy) GetWeights() map[string]float64 {
	return map[string]float64{
		"vector": 1.0,
	}
}

type HybridSearchStrategy struct{}

func (s *HybridSearchStrategy) Name() string {
	return "hybrid_search"
}

func (s *HybridSearchStrategy) ShouldRoute(ctx context.Context, req *model.SearchRequest) bool {
	query := strings.TrimSpace(req.Query)
	
	words := strings.Fields(query)
	
	if len(words) >= 3 && len(words) <= 6 {
		return true
	}
	
	if req.EngineConfig != nil && req.EngineConfig.Vector != nil && req.EngineConfig.Vector.Hybrid {
		return true
	}
	
	return false
}

func (s *HybridSearchStrategy) GetEngines() []string {
	return []string{"bm25", "vector"}
}

func (s *HybridSearchStrategy) GetWeights() map[string]float64 {
	return map[string]float64{
		"bm25":   0.5,
		"vector": 0.5,
	}
}

type AutoRoutingStrategy struct{}

func (s *AutoRoutingStrategy) Name() string {
	return "auto_routing"
}

func (s *AutoRoutingStrategy) ShouldRoute(ctx context.Context, req *model.SearchRequest) bool {
	return true
}

func (s *AutoRoutingStrategy) GetEngines() []string {
	return []string{"flexsearch", "bm25", "vector"}
}

func (s *AutoRoutingStrategy) GetWeights() map[string]float64 {
	return map[string]float64{
		"flexsearch": 0.3,
		"bm25":       0.3,
		"vector":     0.4,
	}
}

type RoutingDecision struct {
	StrategyName string
	Engines      []string
	Weights      map[string]float64
	QueryInfo    *model.QueryInfo
	Timestamp    time.Time
}

func NewRouter(logger *util.Logger) *Router {
	r := &Router{
		logger:  logger,
		strategies: make(map[string]RoutingStrategy),
	}
	
	r.strategies["exact_match"] = &ExactMatchStrategy{}
	r.strategies["fuzzy_search"] = &FuzzySearchStrategy{}
	r.strategies["semantic_search"] = &SemanticSearchStrategy{}
	r.strategies["hybrid_search"] = &HybridSearchStrategy{}
	r.strategies["auto_routing"] = &AutoRoutingStrategy{}
	
	return r
}

func (r *Router) Route(ctx context.Context, req *model.SearchRequest) *RoutingDecision {
	queryInfo := r.analyzeQuery(req)
	
	var selectedStrategy RoutingStrategy
	
	if len(req.Engines) > 0 {
		selectedStrategy = &AutoRoutingStrategy{}
	} else {
		for _, strategy := range r.strategies {
			if strategy.ShouldRoute(ctx, req) {
				selectedStrategy = strategy
				break
			}
		}
	}
	
	if selectedStrategy == nil {
		selectedStrategy = &AutoRoutingStrategy{}
	}
	
	decision := &RoutingDecision{
		StrategyName: selectedStrategy.Name(),
		Engines:      selectedStrategy.GetEngines(),
		Weights:      selectedStrategy.GetWeights(),
		QueryInfo:    queryInfo,
		Timestamp:    time.Now(),
	}
	
	r.logger.Infow("Routing decision made",
		"query", req.Query,
		"strategy", decision.StrategyName,
		"engines", decision.Engines,
		"query_type", queryInfo.QueryType,
	)
	
	return decision
}

func (r *Router) analyzeQuery(req *model.SearchRequest) *model.QueryInfo {
	query := strings.TrimSpace(req.Query)
	
	queryInfo := &model.QueryInfo{
		Query:       query,
		QueryLength: len(query),
		Timestamp:   time.Now(),
	}
	
	words := strings.Fields(query)
	
	if len(words) == 0 {
		queryInfo.QueryType = "empty"
		return queryInfo
	}
	
	if len(words) == 1 {
		queryInfo.QueryType = "single_term"
	} else if len(words) <= 3 {
		queryInfo.QueryType = "short_phrase"
	} else if len(words) <= 6 {
		queryInfo.QueryType = "medium_phrase"
	} else {
		queryInfo.QueryType = "long_query"
	}
	
	queryInfo.HasWildcard = strings.ContainsAny(query, "*?")
	queryInfo.HasPhrase = strings.Contains(query, "\"")
	queryInfo.HasBoolean = detectBooleanOperators(query)
	queryInfo.HasSpecial = detectSpecialCharacters(query)
	
	return queryInfo
}

func detectPotentialTypos(query string) bool {
	words := strings.Fields(query)
	for _, word := range words {
		if len(word) > 3 {
			consecutiveConsonants := 0
			for i := 0; i < len(word); i++ {
				c := strings.ToLower(string(word[i]))
				if !strings.ContainsAny(c, "aeiou") {
					consecutiveConsonants++
					if consecutiveConsonants >= 4 {
						return true
					}
				} else {
					consecutiveConsonants = 0
				}
			}
		}
	}
	return false
}

func containsStopWords(query string) bool {
	stopWords := []string{"the", "a", "an", "is", "are", "was", "were", "be", "been", "being", 
		"have", "has", "had", "do", "does", "did", "will", "would", "could", "should", 
		"may", "might", "must", "shall", "can", "need", "dare", "ought", "used", "to", 
		"of", "in", "for", "on", "with", "at", "by", "from", "as", "into", "through", 
		"during", "before", "after", "above", "below", "between", "under", "again", 
		"further", "then", "once"}
	
	queryLower := strings.ToLower(query)
	for _, stopWord := range stopWords {
		if strings.Contains(queryLower, " "+stopWord+" ") || 
		   strings.HasPrefix(queryLower, stopWord+" ") || 
		   strings.HasSuffix(queryLower, " "+stopWord) {
			return true
		}
	}
	return false
}

func detectBooleanOperators(query string) bool {
	operators := []string{"AND", "OR", "NOT", "&&", "||", "!"}
	queryUpper := strings.ToUpper(query)
	
	for _, op := range operators {
		if strings.Contains(queryUpper, op) {
			return true
		}
	}
	return false
}

func detectSpecialCharacters(query string) bool {
	specialChars := regexp.MustCompile(`[^\w\s\*\?\"\-]`)
	return specialChars.MatchString(query)
}
