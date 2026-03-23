package util

import (
	"context"
	"errors"
	"sync"
	"sync/atomic"
	"time"
)

// CircuitBreakerState represents the state of the circuit breaker
type CircuitBreakerState int32

const (
	StateClosed CircuitBreakerState = iota
	StateOpen
	StateHalfOpen
)

// CircuitBreakerConfig holds the configuration for the circuit breaker
type CircuitBreakerConfig struct {
	FailureThreshold    int           // Number of failures before opening circuit
	SuccessThreshold    int           // Number of successes before closing circuit from half-open
	Timeout             time.Duration // Time to wait before transitioning from open to half-open
	MinRequestThreshold int           // Minimum number of requests before evaluating failures
}

// DefaultCircuitBreakerConfig returns a default configuration
func DefaultCircuitBreakerConfig() CircuitBreakerConfig {
	return CircuitBreakerConfig{
		FailureThreshold:    5,
		SuccessThreshold:    2,
		Timeout:             30 * time.Second,
		MinRequestThreshold: 10,
	}
}

// CircuitBreaker implements the circuit breaker pattern
type CircuitBreaker struct {
	name         string
	config       CircuitBreakerConfig
	state        int32
	failures     int32
	successes    int32
	requests     int32
	lastFailTime time.Time
	mutex        sync.RWMutex
}

// NewCircuitBreaker creates a new circuit breaker
func NewCircuitBreaker(name string, config CircuitBreakerConfig) *CircuitBreaker {
	return &CircuitBreaker{
		name:   name,
		config: config,
		state:  int32(StateClosed),
	}
}

// Execute runs the given function with circuit breaker protection
func (cb *CircuitBreaker) Execute(ctx context.Context, fn func() error) error {
	if !cb.allowRequest() {
		return errors.New("circuit breaker is open")
	}

	err := fn()
	cb.recordResult(err)
	return err
}

// allowRequest checks if a request should be allowed
func (cb *CircuitBreaker) allowRequest() bool {
	state := cb.getState()

	switch state {
	case StateClosed:
		return true
	case StateOpen:
		if time.Since(cb.lastFailTime) > cb.config.Timeout {
			cb.setState(StateHalfOpen)
			return true
		}
		return false
	case StateHalfOpen:
		return true
	default:
		return false
	}
}

// recordResult records the result of a request
func (cb *CircuitBreaker) recordResult(err error) {
	atomic.AddInt32(&cb.requests, 1)

	if err != nil {
		atomic.AddInt32(&cb.failures, 1)
		cb.lastFailTime = time.Now()
		cb.onFailure()
	} else {
		atomic.AddInt32(&cb.successes, 1)
		cb.onSuccess()
	}
}

// onFailure handles failure logic
func (cb *CircuitBreaker) onFailure() {
	state := cb.getState()

	switch state {
	case StateClosed:
		if cb.getFailures() >= int32(cb.config.FailureThreshold) &&
			cb.getRequests() >= int32(cb.config.MinRequestThreshold) {
			cb.setState(StateOpen)
		}
	case StateHalfOpen:
		cb.setState(StateOpen)
	}
}

// onSuccess handles success logic
func (cb *CircuitBreaker) onSuccess() {
	state := cb.getState()

	switch state {
	case StateHalfOpen:
		if cb.getSuccesses() >= int32(cb.config.SuccessThreshold) {
			cb.reset()
			cb.setState(StateClosed)
		}
	}
}

// getState returns the current state
func (cb *CircuitBreaker) getState() CircuitBreakerState {
	return CircuitBreakerState(atomic.LoadInt32(&cb.state))
}

// setState sets the circuit breaker state
func (cb *CircuitBreaker) setState(state CircuitBreakerState) {
	atomic.StoreInt32(&cb.state, int32(state))
}

// getFailures returns the number of failures
func (cb *CircuitBreaker) getFailures() int32 {
	return atomic.LoadInt32(&cb.failures)
}

// getSuccesses returns the number of successes
func (cb *CircuitBreaker) getSuccesses() int32 {
	return atomic.LoadInt32(&cb.successes)
}

// getRequests returns the total number of requests
func (cb *CircuitBreaker) getRequests() int32 {
	return atomic.LoadInt32(&cb.requests)
}

// reset resets the circuit breaker counters
func (cb *CircuitBreaker) reset() {
	atomic.StoreInt32(&cb.failures, 0)
	atomic.StoreInt32(&cb.successes, 0)
	atomic.StoreInt32(&cb.requests, 0)
}

// GetState returns the current state (for monitoring)
func (cb *CircuitBreaker) GetState() string {
	state := cb.getState()
	switch state {
	case StateClosed:
		return "closed"
	case StateOpen:
		return "open"
	case StateHalfOpen:
		return "half-open"
	default:
		return "unknown"
	}
}

// GetStats returns circuit breaker statistics
func (cb *CircuitBreaker) GetStats() map[string]interface{} {
	return map[string]interface{}{
		"name":      cb.name,
		"state":     cb.GetState(),
		"failures":  cb.getFailures(),
		"successes": cb.getSuccesses(),
		"requests":  cb.getRequests(),
	}
}
