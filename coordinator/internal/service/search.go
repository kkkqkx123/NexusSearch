package service

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/flexsearch/coordinator/internal/cache"
	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/engine"
	"github.com/flexsearch/coordinator/internal/merger"
	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/router"
	"github.com/flexsearch/coordinator/internal/util"
)

type SearchService struct {
	config        *config.Config
	logger        *util.Logger
	cache         *cache.RedisCache
	router        *router.Router
	optimizer     *router.Optimizer
	merger        merger.Merger
	engines       map[string]engine.EngineClient
	metrics       *util.Metrics
}

type SearchServiceConfig struct {
	Config       *config.Config
	Logger       *util.Logger
	Cache        *cache.RedisCache
	Router       *router.Router
	Optimizer    *router.Optimizer
	Merger       merger.Merger
	Engines      map[string]engine.EngineClient
	Metrics      *util.Metrics
}

func NewSearchService(cfg *SearchServiceConfig) *SearchService {
	return &SearchService{
		config:    cfg.Config,
		logger:    cfg.Logger,
		cache:     cfg.Cache,
		router:    cfg.Router,
		optimizer: cfg.Optimizer,
		merger:    cfg.Merger,
		engines:   cfg.Engines,
		metrics:   cfg.Metrics,
	}
}

func (s *SearchService) Search(ctx context.Context, req *model.SearchRequest) (*model.SearchResponse, error) {
	startTime := time.Now()
	
	if req.RequestID == "" {
		req.RequestID = generateRequestID()
	}

	s.logger.Infow("Search request received",
		"request_id", req.RequestID,
		"query", req.Query,
		"index", req.Index,
	)

	if s.cache != nil && s.cache.IsEnabled() {
		cached, found := s.cache.GetSearchResponse(ctx, req)
		if found {
			s.logger.Infow("Cache hit",
				"request_id", req.RequestID,
				"took_ms", time.Since(startTime).Milliseconds(),
			)
			s.metrics.RecordCacheHit()
			return cached, nil
		}
		s.metrics.RecordCacheMiss()
	}

	optimized := s.optimizer.Optimize(ctx, req)
	if optimized.Rewritten {
		s.logger.Debugw("Query rewritten",
			"original", optimized.OriginalQuery,
			"rewritten", optimized.RewrittenQuery,
		)
	}

	searchReq := *req
	searchReq.Query = optimized.RewrittenQuery

	decision := s.router.Route(ctx, &searchReq)
	
	results, err := s.executeSearch(ctx, &searchReq, decision)
	if err != nil {
		s.logger.Errorf("Search execution failed: %v", err)
		return s.handleError(ctx, req, err), nil
	}

	response := s.merger.Merge(results)
	response.RequestID = req.RequestID
	response.QueryInfo = decision.QueryInfo
	response.CacheHit = false

	if s.cache != nil && s.cache.IsEnabled() {
		go s.cache.SetSearchResponse(context.Background(), req, response, s.config.Cache.DefaultTTL)
	}

	totalTime := time.Since(startTime)
	s.logger.Infow("Search completed",
		"request_id", req.RequestID,
		"results", len(response.Results),
		"engines", response.EnginesUsed,
		"took_ms", totalTime.Milliseconds(),
	)

	s.metrics.RecordSearchDuration(float64(totalTime.Milliseconds()))
	s.metrics.RecordSearchResults(len(response.Results))

	return response, nil
}

func (s *SearchService) executeSearch(ctx context.Context, req *model.SearchRequest, decision *router.RoutingDecision) (map[string]*model.EngineResult, error) {
	timeout := 800 * time.Millisecond
	if req.Timeout > 0 {
		timeout = req.Timeout
	}

	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	results := make(map[string]*model.EngineResult)
	var mu sync.Mutex
	var wg sync.WaitGroup
	var hasError bool

	for _, engineName := range decision.Engines {
		client, exists := s.engines[engineName]
		if !exists {
			s.logger.Warnf("Engine %s not configured", engineName)
			continue
		}

		wg.Add(1)
		go func(name string, client engine.EngineClient) {
			defer wg.Done()

			result, err := client.Search(ctx, req)
			
			mu.Lock()
			defer mu.Unlock()

			if err != nil {
				s.logger.Warnw("Engine search failed",
					"engine", name,
					"error", err,
				)
				results[name] = &model.EngineResult{
					Engine:   name,
					Results:  []model.SearchResult{},
					Total:    0,
					Took:     0,
					Error:    err.Error(),
					TimedOut: ctx.Err() == context.DeadlineExceeded,
				}
				hasError = true
			} else {
				results[name] = result
			}
		}(engineName, client)
	}

	wg.Wait()

	if len(results) == 0 {
		return nil, fmt.Errorf("no engines available")
	}

	if hasError && len(results) > 1 {
		s.logger.Warnw("Some engines failed, continuing with available results",
			"total_engines", len(decision.Engines),
			"successful", len(results),
		)
	}

	return results, nil
}

func (s *SearchService) handleError(ctx context.Context, req *model.SearchRequest, err error) *model.SearchResponse {
	response := &model.SearchResponse{
		RequestID:   req.RequestID,
		Results:     []model.SearchResult{},
		Total:       0,
		Took:        0,
		EnginesUsed: []string{},
		CacheHit:    false,
		QueryInfo: &model.QueryInfo{
			Query:       req.Query,
			QueryLength: len(req.Query),
			Timestamp:   time.Now(),
		},
	}

	s.logger.Errorf("Returning error response: %v", err)
	return response
}

func (s *SearchService) HealthCheck(ctx context.Context) map[string]bool {
	health := make(map[string]bool)
	
	for name, client := range s.engines {
		health[name] = client.HealthCheck(ctx)
	}
	
	return health
}

func (s *SearchService) GetCacheStats() *model.CacheStats {
	if s.cache == nil {
		return &model.CacheStats{}
	}
	return s.cache.GetStats()
}

func (s *SearchService) ClearCache(ctx context.Context) error {
	if s.cache == nil {
		return nil
	}
	return s.cache.Clear(ctx)
}

func (s *SearchService) WarmupCache(ctx context.Context, queries []string, index string) error {
	if s.cache == nil {
		return nil
	}
	return s.cache.Warmup(ctx, queries, index)
}

func generateRequestID() string {
	return fmt.Sprintf("req-%d", time.Now().UnixNano())
}
