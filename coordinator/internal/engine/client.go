package engine

import (
	"context"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
)

type EngineClient interface {
	Connect(ctx context.Context) error
	Disconnect() error
	Search(ctx context.Context, req *model.SearchRequest) (*model.EngineResult, error)
	HealthCheck(ctx context.Context) bool
	GetName() string
}

type ClientConfig struct {
	Host       string
	Port       int
	Timeout    time.Duration
	MaxRetries int
	PoolSize   int
}

type RetryConfig struct {
	MaxRetries int
	InitialDelay time.Duration
	MaxDelay     time.Duration
	BackoffFactor float64
}

type CircuitBreakerConfig struct {
	FailureThreshold int
	SuccessThreshold int
	Timeout          time.Duration
}

type CircuitBreakerState int

const (
	StateClosed CircuitBreakerState = iota
	StateOpen
	StateHalfOpen
)

type CircuitBreaker struct {
	state         CircuitBreakerState
	failureCount  int
	successCount  int
	lastFailTime  time.Time
	config        *CircuitBreakerConfig
}

func NewCircuitBreaker(config *CircuitBreakerConfig) *CircuitBreaker {
	return &CircuitBreaker{
		state: StateClosed,
		config: config,
	}
}

func (cb *CircuitBreaker) AllowRequest() bool {
	switch cb.state {
	case StateClosed:
		return true
	case StateOpen:
		if time.Since(cb.lastFailTime) > cb.config.Timeout {
			cb.state = StateHalfOpen
			cb.successCount = 0
			return true
		}
		return false
	case StateHalfOpen:
		return true
	default:
		return false
	}
}

func (cb *CircuitBreaker) RecordSuccess() {
	switch cb.state {
	case StateClosed:
		cb.failureCount = 0
	case StateHalfOpen:
		cb.successCount++
		if cb.successCount >= cb.config.SuccessThreshold {
			cb.state = StateClosed
			cb.failureCount = 0
		}
	}
}

func (cb *CircuitBreaker) RecordFailure() {
	cb.failureCount++
	cb.lastFailTime = time.Now()

	if cb.failureCount >= cb.config.FailureThreshold {
		cb.state = StateOpen
	}
}

func (cb *CircuitBreaker) GetState() CircuitBreakerState {
	return cb.state
}

func (cb *CircuitBreaker) GetFailureCount() int {
	return cb.failureCount
}
