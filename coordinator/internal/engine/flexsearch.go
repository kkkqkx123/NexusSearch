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

type FlexSearchClient struct {
	config       *ClientConfig
	conn         *grpc.ClientConn
	logger       *util.Logger
	circuitBreaker *CircuitBreaker
	retryConfig  *RetryConfig
}

func NewFlexSearchClient(config *ClientConfig, logger *util.Logger) *FlexSearchClient {
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

	return &FlexSearchClient{
		config:        config,
		logger:        logger,
		circuitBreaker: NewCircuitBreaker(cbConfig),
		retryConfig:   retryConfig,
	}
}

func (c *FlexSearchClient) Connect(ctx context.Context) error {
	address := fmt.Sprintf("%s:%d", c.config.Host, c.config.Port)
	
	conn, err := grpc.Dial(address, 
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithDefaultCallOptions(
			grpc.MaxCallRecvMsgSize(100*1024*1024),
			grpc.MaxCallSendMsgSize(100*1024*1024),
		),
	)
	if err != nil {
		return fmt.Errorf("failed to connect to FlexSearch: %w", err)
	}

	c.conn = conn
	c.logger.Infof("FlexSearch client connected to %s", address)
	return nil
}

func (c *FlexSearchClient) Disconnect() error {
	if c.conn != nil {
		err := c.conn.Close()
		c.conn = nil
		c.logger.Info("FlexSearch client disconnected")
		return err
	}
	return nil
}

func (c *FlexSearchClient) Search(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	if !c.circuitBreaker.AllowRequest() {
		return nil, fmt.Errorf("circuit breaker is open for FlexSearch")
	}

	result, err := c.searchWithRetry(ctx, req)
	
	if err != nil {
		c.circuitBreaker.RecordFailure()
		c.logger.Errorf("FlexSearch search failed: %v", err)
		return nil, err
	}

	c.circuitBreaker.RecordSuccess()
	return result, nil
}

func (c *FlexSearchClient) searchWithRetry(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	var lastErr error
	
	for attempt := 0; attempt <= c.retryConfig.MaxRetries; attempt++ {
		if attempt > 0 {
			delay := c.calculateBackoff(attempt)
			c.logger.Debugf("FlexSearch retry attempt %d after %v", attempt, delay)
			
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

	return nil, fmt.Errorf("FlexSearch search failed after %d retries: %w", c.retryConfig.MaxRetries, lastErr)
}

func (c *FlexSearchClient) doSearch(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error) {
	startTime := time.Now()
	
	timeout := c.config.Timeout
	if req.Timeout > 0 {
		timeout = req.Timeout
	}
	
	ctx, cancel := context.WithTimeout(ctx, timeout)
	defer cancel()

	result := &model.EngineResult{
		Engine:  "flexsearch",
		Results: []model.SearchResult{},
		Total:   0,
		Took:    0,
	}

	for i := 0; i < int(req.Limit); i++ {
		score := 1.0 - float64(i)*0.1
		if score < 0 {
			score = 0
		}
		
		result.Results = append(result.Results, model.SearchResult{
			ID:           c.generateID(req.Query, i),
			Index:        req.Index,
			Score:        score,
			Title:        fmt.Sprintf("FlexSearch Result %d for: %s", i+1, req.Query),
			Content:      fmt.Sprintf("Sample content from FlexSearch for query: %s", req.Query),
			EngineSource: "flexsearch",
			Rank:         int32(i + 1),
		})
	}

	result.Total = int64(len(result.Results))
	result.Took = float64(time.Since(startTime).Milliseconds())

	c.logger.Debugf("FlexSearch returned %d results in %.2fms", result.Total, result.Took)
	return result, nil
}

func (c *FlexSearchClient) HealthCheck(ctx context.Context) bool {
	if c.conn == nil {
		return false
	}

	ctx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	state := c.conn.GetState()
	return state == connectivity.Ready || state == connectivity.Idle
}

func (c *FlexSearchClient) GetName() string {
	return "flexsearch"
}

func (c *FlexSearchClient) isRetryableError(err error) bool {
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

func (c *FlexSearchClient) calculateBackoff(attempt int) time.Duration {
	delay := float64(c.retryConfig.InitialDelay) * math.Pow(c.retryConfig.BackoffFactor, float64(attempt-1))
	
	if delay > float64(c.retryConfig.MaxDelay) {
		delay = float64(c.retryConfig.MaxDelay)
	}
	
	return time.Duration(delay)
}

func (c *FlexSearchClient) generateID(query string, index int) string {
	h := md5.New()
	h.Write([]byte(fmt.Sprintf("%s-%d", query, index)))
	return hex.EncodeToString(h.Sum(nil))[:16]
}
