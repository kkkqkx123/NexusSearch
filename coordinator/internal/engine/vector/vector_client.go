package vector

import (
	"context"
	"fmt"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/qdrant/go-client/qdrant"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

type ClientConfig struct {
	Host        string
	Port        int
	Timeout     time.Duration
	MaxRetries  int
	PoolSize    int
	APIKey      string
	UseTLS      bool
	GRPCOptions []grpc.DialOption
}

type VectorClient struct {
	config           *ClientConfig
	vectorConfig     *VectorEngineConfig
	qdrantClient     *qdrant.Client
	embeddingService *EmbeddingService
	logger           *util.Logger
	circuitBreaker   *CircuitBreaker
	retryConfig      *RetryConfig
}

type VectorEngineConfig struct {
	Model               string
	Dimension           int
	Threshold           float64
	TopK                int
	Hybrid              bool
	Alpha               float64
	Collection          string
	EmbeddingServiceURL string
}

type RetryConfig struct {
	MaxRetries    int
	InitialDelay  time.Duration
	MaxDelay      time.Duration
	BackoffFactor float64
}

type CircuitBreakerConfig struct {
	FailureThreshold int
	SuccessThreshold int
	Timeout          time.Duration
}

type CircuitBreaker struct {
	config          *CircuitBreakerConfig
	failures        int
	successes       int
	state           CircuitBreakerState
	lastFailureTime time.Time
}

type CircuitBreakerState int

const (
	CircuitBreakerClosed CircuitBreakerState = iota
	CircuitBreakerOpen
	CircuitBreakerHalfOpen
)

func NewCircuitBreaker(config *CircuitBreakerConfig) *CircuitBreaker {
	return &CircuitBreaker{
		config: config,
		state:  CircuitBreakerClosed,
	}
}

func (cb *CircuitBreaker) AllowRequest() bool {
	if cb.state == CircuitBreakerClosed {
		return true
	}

	if cb.state == CircuitBreakerOpen {
		if time.Since(cb.lastFailureTime) > cb.config.Timeout {
			cb.state = CircuitBreakerHalfOpen
			return true
		}
		return false
	}

	return true
}

func (cb *CircuitBreaker) RecordSuccess() {
	if cb.state == CircuitBreakerHalfOpen {
		cb.successes++
		if cb.successes >= cb.config.SuccessThreshold {
			cb.state = CircuitBreakerClosed
			cb.failures = 0
			cb.successes = 0
		}
	} else if cb.state == CircuitBreakerClosed {
		cb.failures = 0
	}
}

func (cb *CircuitBreaker) RecordFailure() {
	cb.failures++
	cb.lastFailureTime = time.Now()

	if cb.failures >= cb.config.FailureThreshold {
		cb.state = CircuitBreakerOpen
		cb.successes = 0
	}
}

type EmbeddingService struct {
	url       string
	model     string
	dimension int
}

func NewEmbeddingService(url, model string, dimension int) *EmbeddingService {
	return &EmbeddingService{
		url:       url,
		model:     model,
		dimension: dimension,
	}
}

func (e *EmbeddingService) GenerateEmbedding(ctx context.Context, text string) ([]float64, error) {
	return make([]float64, e.dimension), nil
}

func NewVectorClient(config *ClientConfig, vectorConfig *VectorEngineConfig, logger *util.Logger) (*VectorClient, error) {
	if vectorConfig == nil {
		return nil, fmt.Errorf("vectorConfig cannot be nil")
	}
	if vectorConfig.Dimension <= 0 {
		return nil, fmt.Errorf("vector dimension must be positive, got %d", vectorConfig.Dimension)
	}
	if vectorConfig.Threshold < 0 || vectorConfig.Threshold > 1 {
		return nil, fmt.Errorf("vector threshold must be between 0 and 1, got %f", vectorConfig.Threshold)
	}
	if vectorConfig.TopK <= 0 {
		return nil, fmt.Errorf("vector TopK must be positive, got %d", vectorConfig.TopK)
	}

	cbConfig := &CircuitBreakerConfig{
		FailureThreshold: 5,
		SuccessThreshold: 2,
		Timeout:          30 * time.Second,
	}

	retryConfig := &RetryConfig{
		MaxRetries:    config.MaxRetries,
		InitialDelay:  100 * time.Millisecond,
		MaxDelay:      5 * time.Second,
		BackoffFactor: 2.0,
	}

	return &VectorClient{
		config:         config,
		vectorConfig:   vectorConfig,
		logger:         logger,
		circuitBreaker: NewCircuitBreaker(cbConfig),
		retryConfig:    retryConfig,
	}, nil
}

func (c *VectorClient) Connect(ctx context.Context) error {
	address := fmt.Sprintf("%s:%d", c.config.Host, c.config.Port)

	client, err := qdrant.NewClient(&qdrant.Config{
		Host: c.config.Host,
		Port: c.config.Port,
	})
	if err != nil {
		return fmt.Errorf("failed to create Qdrant client: %w", err)
	}

	c.qdrantClient = client

	if c.vectorConfig.EmbeddingServiceURL != "" {
		c.embeddingService = NewEmbeddingService(
			c.vectorConfig.EmbeddingServiceURL,
			c.vectorConfig.Model,
			c.vectorConfig.Dimension,
		)
	}

	c.logger.Infof("Vector client connected to Qdrant at %s", address)
	return nil
}

func (c *VectorClient) Disconnect() error {
	if c.qdrantClient != nil {
		if err := c.qdrantClient.Close(); err != nil {
			c.logger.Warnf("Error closing Qdrant client: %v", err)
		}
		c.qdrantClient = nil
		c.logger.Info("Vector client disconnected")
	}
	return nil
}

func (c *VectorClient) Search(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	if !c.circuitBreaker.AllowRequest() {
		return nil, fmt.Errorf("circuit breaker is open for Vector")
	}

	result, err := c.searchWithRetry(ctx, req)

	if err != nil {
		c.circuitBreaker.RecordFailure()
		c.logger.Errorf("Vector search failed: %v", err)
		return nil, err
	}

	c.circuitBreaker.RecordSuccess()
	return result, nil
}

func (c *VectorClient) searchWithRetry(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	var lastErr error

	for attempt := 0; attempt <= c.retryConfig.MaxRetries; attempt++ {
		if attempt > 0 {
			delay := c.calculateBackoff(attempt)
			c.logger.Debugf("Vector retry attempt %d after %v", attempt, delay)

			select {
			case <-time.After(delay):
			case <-ctx.Done():
				return nil, ctx.Err()
			}
		}

		result, err := c.doSearch(ctx, req)
		if err == nil {
			return result, nil
		}

		lastErr = err

		if !c.isRetryableError(err) {
			break
		}
	}

	return nil, fmt.Errorf("Vector search failed after %d retries: %w", c.retryConfig.MaxRetries, lastErr)
}

func (c *VectorClient) doSearch(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	startTime := time.Now()

	timeout := c.config.Timeout
	if req.Timeout > 0 {
		timeout = req.Timeout
	}

	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	queryVector, err := c.getQueryVector(ctx, req.Query)
	if err != nil {
		return nil, fmt.Errorf("failed to get query vector: %w", err)
	}

	collection := c.getCollection(req.Index)
	topK := c.getTopK()
	if topK <= 0 {
		topK = int(req.Limit)
	}

	searchResult, err := c.qdrantClient.Query(ctx, &qdrant.QueryPoints{
		CollectionName: collection,
		Query:          qdrant.NewQuery(queryVector...),
		Limit:          qdrant.PtrOf(uint64(topK)),
		WithPayload:    qdrant.NewWithPayload(true),
		ScoreThreshold: qdrant.PtrOf(float32(c.vectorConfig.Threshold)),
	})
	if err != nil {
		return nil, fmt.Errorf("Qdrant search failed: %w", err)
	}

	result := &model.EngineResult{
		Engine:  "vector",
		Results: make([]model.SearchResult, 0, len(searchResult)),
		Total:   int64(len(searchResult)),
		Took:    float64(time.Since(startTime).Milliseconds()),
	}

	for i, point := range searchResult {
		searchResult := model.SearchResult{
			ID:           fmt.Sprintf("%v", point.GetId()),
			Index:        req.Index,
			Score:        float64(point.GetScore()),
			EngineSource: "vector",
			Rank:         int32(i + 1),
		}

		payload := point.GetPayload()
		if payload != nil {
			searchResult.Title = c.extractPayloadField(payload, "title")
			searchResult.Content = c.extractPayloadField(payload, "content")
		}

		result.Results = append(result.Results, searchResult)
	}

	c.logger.Debugf("Vector returned %d results in %.2fms", result.Total, result.Took)
	return result, nil
}

func (c *VectorClient) getQueryVector(ctx context.Context, query string) ([]float32, error) {
	if c.embeddingService != nil {
		embedding, err := c.embeddingService.GenerateEmbedding(ctx, query)
		if err != nil {
			c.logger.Warnf("Failed to generate embedding, using fallback: %v", err)
			return c.generateFallbackVector(query), nil
		}
		return c.float64ToFloat32(embedding), nil
	}

	return c.generateFallbackVector(query), nil
}

func (c *VectorClient) generateFallbackVector(query string) []float32 {
	dimension := c.vectorConfig.Dimension
	vector := make([]float32, dimension)

	hash := 0
	for _, ch := range query {
		hash = hash*31 + int(ch)
	}

	for i := 0; i < dimension; i++ {
		vector[i] = float32((hash+i)%256) / 255.0
	}

	return vector
}

func (c *VectorClient) float64ToFloat32(slice []float64) []float32 {
	result := make([]float32, len(slice))
	for i, v := range slice {
		result[i] = float32(v)
	}
	return result
}

func (c *VectorClient) getCollection(index string) string {
	if c.vectorConfig.Collection != "" {
		return c.vectorConfig.Collection
	}
	if index != "" {
		return index
	}
	return "default"
}

func (c *VectorClient) extractPayloadField(payload map[string]*qdrant.Value, field string) string {
	if val, ok := payload[field]; ok {
		return val.GetStringValue()
	}
	return ""
}

func (c *VectorClient) HealthCheck(ctx context.Context) bool {
	if c.qdrantClient == nil {
		return false
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	_, err := c.qdrantClient.HealthCheck(ctx)
	return err == nil
}

func (c *VectorClient) GetName() string {
	return "vector"
}

func (c *VectorClient) getTopK() int {
	return c.vectorConfig.TopK
}

func (c *VectorClient) isRetryableError(err error) bool {
	if err == nil {
		return false
	}

	st, ok := status.FromError(err)
	if !ok {
		return false
	}

	switch st.Code() {
	case codes.DeadlineExceeded, codes.Unavailable, codes.Aborted, codes.ResourceExhausted:
		return true
	default:
		return false
	}
}

func (c *VectorClient) calculateBackoff(attempt int) time.Duration {
	delay := float64(c.retryConfig.InitialDelay)
	for i := 1; i < attempt; i++ {
		delay *= c.retryConfig.BackoffFactor
	}

	if delay > float64(c.retryConfig.MaxDelay) {
		delay = float64(c.retryConfig.MaxDelay)
	}

	return time.Duration(delay)
}

func extractValue(v *qdrant.Value) interface{} {
	if v == nil {
		return nil
	}

	switch val := v.Kind.(type) {
	case *qdrant.Value_StringValue:
		return val.StringValue
	case *qdrant.Value_IntegerValue:
		return val.IntegerValue
	case *qdrant.Value_DoubleValue:
		return val.DoubleValue
	case *qdrant.Value_BoolValue:
		return val.BoolValue
	case *qdrant.Value_ListValue:
		if val.ListValue != nil {
			result := make([]interface{}, len(val.ListValue.Values))
			for i, item := range val.ListValue.Values {
				result[i] = extractValue(item)
			}
			return result
		}
	case *qdrant.Value_StructValue:
		if val.StructValue != nil {
			result := make(map[string]interface{})
			for k, v := range val.StructValue.Fields {
				result[k] = extractValue(v)
			}
			return result
		}
	}

	return nil
}
