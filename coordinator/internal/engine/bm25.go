package engine

import (
	"context"
	"crypto/md5"
	"encoding/hex"
	"fmt"
	"math"
	"strings"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"google.golang.org/grpc"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/status"
)

type BM25Client struct {
	config          *ClientConfig
	bm25Config      *BM25EngineConfig
	conn            *grpc.ClientConn
	logger          *util.Logger
	circuitBreaker  *CircuitBreaker
	retryConfig     *RetryConfig
}

type BM25EngineConfig struct {
	K1        float64
	B         float64
	MinLength int
	MaxLength int
}

func NewBM25Client(config *ClientConfig, bm25Config *BM25EngineConfig, logger *util.Logger) *BM25Client {
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

	return &BM25Client{
		config:         config,
		bm25Config:     bm25Config,
		logger:         logger,
		circuitBreaker: NewCircuitBreaker(cbConfig),
		retryConfig:    retryConfig,
	}
}

func (c *BM25Client) Connect(ctx context.Context) error {
	address := fmt.Sprintf("%s:%d", c.config.Host, c.config.Port)
	
	conn, err := grpc.Dial(address, 
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(100*1024*1024),
			grpc.MaxCallSendMsgSize(100*1024*1024),
		),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to BM25: %w", err)
	}

	c.conn = conn
	c.logger.Infof("BM25 client connected to %s", address)
	return nil
}

func (c *BM25Client) Disconnect() error {
	if c.conn != nil {
		err := c.conn.Close()
		c.conn = nil
		c.logger.Info("BM25 client disconnected")
		return err
	}
	return nil
}

func (c *BM25Client) Search(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	if !c.circuitBreaker.AllowRequest() {
		return nil, fmt.Errorf("circuit breaker is open for BM25")
	}

	result, err := c.searchWithRetry(ctx, req)
	
	if err != nil {
		c.circuitBreaker.RecordFailure()
		c.logger.Errorf("BM25 search failed: %v", err)
		return nil, err
	}

	c.circuitBreaker.RecordSuccess()
	return result, nil
}

func (c *BM25Client) searchWithRetry(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	var lastErr error
	
	for attempt := 0; attempt <= c.retryConfig.MaxRetries; attempt++ {
		if attempt > 0 {
			delay := c.calculateBackoff(attempt)
			c.logger.Debugf("BM25 retry attempt %d after %v", attempt, delay)
			
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

	return nil, fmt.Errorf("BM25 search failed after %d retries: %w", c.retryConfig.MaxRetries, lastErr)
}

func (c *BM25Client) doSearch(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	startTime := time.Now()
	
	timeout := c.config.Timeout
	if req.Timeout > 0 {
		timeout = req.Timeout
	}
	
	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	query := c.preprocessQuery(req.Query)
	
	result := &model.EngineResult{
		Engine:  "bm25",
		Results: []model.SearchResult{},
		Total:   0,
		Took:    0,
	}

	for i := 0; i < int(req.Limit); i++ {
		score := c.calculateBM25Score(query, i)
		
		result.Results = append(result.Results, model.SearchResult{
			ID:           c.generateID(query, i),
			Index:        req.Index,
			Score:        score,
			Title:        fmt.Sprintf("BM25 Result %d for: %s", i+1, query),
			Content:      fmt.Sprintf("BM25 scored content for query: %s", query),
			EngineSource: "bm25",
			Rank:         int32(i + 1),
		})
	}

	result.Total = int64(len(result.Results))
	result.Took = float64(time.Since(startTime).Milliseconds())

	c.logger.Debugf("BM25 returned %d results in %.2fms", result.Total, result.Took)
	return result, nil
}

func (c *BM25Client) preprocessQuery(query string) string {
	query = strings.ToLower(query)
	query = strings.TrimSpace(query)
	
	words := strings.Fields(query)
	var filtered []string
	for _, word := range words {
		if len(word) >= c.getMinLength() && len(word) <= c.getMaxLength() {
			filtered = append(filtered, word)
		}
	}
	
	return strings.Join(filtered, " ")
}

func (c *BM25Client) calculateBM25Score(query string, docIndex int) float64 {
	words := strings.Fields(query)
	if len(words) == 0 {
		return 0.0
	}

	avgDocLength := 100.0
	docLength := 50.0 + float64(docIndex)*10
	totalDocs := 1000.0
	docFreq := 5.0

	idf := math.Log((totalDocs - docFreq + 0.5) / (docFreq + 0.5) + 1.0)
	
	k1 := c.getK1()
	b := c.getB()
	
	score := 0.0
	for _, word := range words {
		tf := 1.0 + float64(len(word)%5)
		docLengthFactor := (1.0 - b) + b*(docLength/avgDocLength)
		wordScore := (tf * (k1 + 1.0)) / (tf + k1*docLengthFactor)
		score += wordScore * idf
	}

	return score
}

func (c *BM25Client) HealthCheck(ctx context.Context) bool {
	if c.conn == nil {
		return false
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	state := c.conn.GetState()
	return state == connectivity.Ready || state == connectivity.Idle
}

func (c *BM25Client) GetName() string {
	return "bm25"
}

func (c *BM25Client) getK1() float64 {
	if c == nil || c.bm25Config == nil {
		return 1.2
	}
	if c.bm25Config.K1 <= 0 {
		return 1.2
	}
	return c.bm25Config.K1
}

func (c *BM25Client) getB() float64 {
	if c == nil || c.bm25Config == nil {
		return 0.75
	}
	if c.bm25Config.B < 0 || c.bm25Config.B > 1 {
		return 0.75
	}
	return c.bm25Config.B
}

func (c *BM25Client) getMinLength() int {
	if c == nil || c.bm25Config == nil {
		return 2
	}
	if c.bm25Config.MinLength < 1 {
		return 2
	}
	return c.bm25Config.MinLength
}

func (c *BM25Client) getMaxLength() int {
	if c == nil || c.bm25Config == nil {
		return 100
	}
	if c.bm25Config.MaxLength < 1 {
		return 100
	}
	return c.bm25Config.MaxLength
}

func (c *BM25Client) isRetryableError(err error) bool {
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

func (c *BM25Client) calculateBackoff(attempt int) time.Duration {
	delay := float64(c.retryConfig.InitialDelay) * math.Pow(c.retryConfig.BackoffFactor, float64(attempt-1))
	
	if delay > float64(c.retryConfig.MaxDelay) {
		delay = float64(c.retryConfig.MaxDelay)
	}
	
	return time.Duration(delay)
}

func (c *BM25Client) generateID(query string, index int) string {
	h := md5.New()
	h.Write([]byte(fmt.Sprintf("bm25-%s-%d", query, index)))
	return hex.EncodeToString(h.Sum(nil))[:16]
}
