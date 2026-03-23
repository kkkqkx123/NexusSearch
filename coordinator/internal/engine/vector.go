package engine

import (
	"context"
	"crypto/md5"
	"encoding/hex"
	"fmt"
	"math"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
)

type VectorClient struct {
	config         *ClientConfig
	vectorConfig   *VectorEngineConfig
	conn           *grpc.ClientConn
	logger         *util.Logger
	circuitBreaker *CircuitBreaker
	retryConfig    *RetryConfig
}

type VectorEngineConfig struct {
	Model     string
	Dimension int
	Threshold float64
	TopK      int
	Hybrid    bool
	Alpha     float64
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

	conn, err := grpc.Dial(address,
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(100*1024*1024),
			grpc.MaxCallSendMsgSize(100*1024*1024),
		),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to Vector: %w", err)
	}

	c.conn = conn
	c.logger.Infof("Vector client connected to %s", address)
	return nil
}

func (c *VectorClient) Disconnect() error {
	if c.conn != nil {
		err := c.conn.Close()
		c.conn = nil
		c.logger.Info("Vector client disconnected")
		return err
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

	queryEmbedding := c.generateEmbedding(req.Query)

	result := &model.EngineResult{
		Engine:  "vector",
		Results: []model.SearchResult{},
		Total:   0,
		Took:    0,
	}

	topK := c.getTopK()
	if topK <= 0 {
		topK = int(req.Limit)
	}

	for i := 0; i < topK; i++ {
		docEmbedding := c.generateDocEmbedding(i)
		similarity := c.calculateCosineSimilarity(queryEmbedding, docEmbedding)

		if similarity >= c.getThreshold() {
			normalizedScore := c.normalizeScore(similarity)

			result.Results = append(result.Results, model.SearchResult{
				ID:           c.generateID(req.Query, i),
				Index:        req.Index,
				Score:        normalizedScore,
				Title:        fmt.Sprintf("Vector Result %d for: %s", i+1, req.Query),
				Content:      fmt.Sprintf("Semantic similarity %.4f for query: %s", similarity, req.Query),
				EngineSource: "vector",
				Rank:         int32(i + 1),
			})
		}
	}

	result.Total = int64(len(result.Results))
	result.Took = float64(time.Since(startTime).Milliseconds())

	c.logger.Debugf("Vector returned %d results in %.2fms", result.Total, result.Took)
	return result, nil
}

func (c *VectorClient) generateEmbedding(query string) []float64 {
	dimension := c.getDimension()
	embedding := make([]float64, dimension)

	hash := md5.Sum([]byte(query))
	for i := 0; i < dimension; i++ {
		if i < len(hash) {
			embedding[i] = float64(hash[i]) / 255.0
		} else {
			embedding[i] = 0.0
		}
	}

	return embedding
}

func (c *VectorClient) generateDocEmbedding(docIndex int) []float64 {
	dimension := c.getDimension()
	embedding := make([]float64, dimension)

	for i := 0; i < dimension; i++ {
		angle := float64(i)*0.1 + float64(docIndex)*0.05
		embedding[i] = math.Sin(angle)*0.5 + 0.5
	}

	return embedding
}

func (c *VectorClient) calculateCosineSimilarity(a, b []float64) float64 {
	if len(a) != len(b) {
		return 0.0
	}

	var dotProduct, normA, normB float64
	for i := 0; i < len(a); i++ {
		dotProduct += a[i] * b[i]
		normA += a[i] * a[i]
		normB += b[i] * b[i]
	}

	if normA == 0 || normB == 0 {
		return 0.0
	}

	return dotProduct / (math.Sqrt(normA) * math.Sqrt(normB))
}

func (c *VectorClient) normalizeScore(score float64) float64 {
	normalized := (score - c.getThreshold()) / (1.0 - c.getThreshold())
	if normalized < 0 {
		return 0.0
	}
	if normalized > 1.0 {
		return 1.0
	}
	return normalized
}

func (c *VectorClient) HealthCheck(ctx context.Context) bool {
	if c.conn == nil {
		return false
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	state := c.conn.GetState()
	return state == connectivity.Ready || state == connectivity.Idle
}

func (c *VectorClient) GetName() string {
	return "vector"
}

func (c *VectorClient) getDimension() int {
	return c.vectorConfig.Dimension
}

func (c *VectorClient) getThreshold() float64 {
	return c.vectorConfig.Threshold
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
	delay := float64(c.retryConfig.InitialDelay) * math.Pow(c.retryConfig.BackoffFactor, float64(attempt-1))

	if delay > float64(c.retryConfig.MaxDelay) {
		delay = float64(c.retryConfig.MaxDelay)
	}

	return time.Duration(delay)
}

func (c *VectorClient) generateID(query string, index int) string {
	h := md5.New()
	h.Write([]byte(fmt.Sprintf("vector-%s-%d", query, index)))
	return hex.EncodeToString(h.Sum(nil))[:16]
}
