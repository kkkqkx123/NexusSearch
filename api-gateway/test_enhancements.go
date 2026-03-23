package main

import (
	"fmt"
	"testing"
	"time"

	"github.com/flexsearch/api-gateway/internal/util"
)

func TestCircuitBreaker(t *testing.T) {
	config := util.DefaultCircuitBreakerConfig()
	config.FailureThreshold = 2
	config.SuccessThreshold = 1
	config.Timeout = 1 * time.Second

	cb := util.NewCircuitBreaker("test-breaker", config)

	// Test initial state
	if cb.GetState() != "closed" {
		t.Errorf("Expected initial state to be closed, got %s", cb.GetState())
	}

	// Test successful execution
	err := cb.Execute(nil, func() error {
		return nil
	})
	if err != nil {
		t.Errorf("Expected no error, got %v", err)
	}

	// Test failure execution
	for i := 0; i < config.FailureThreshold; i++ {
		err = cb.Execute(nil, func() error {
			return fmt.Errorf("test error")
		})
		if err == nil {
			t.Errorf("Expected error, got nil")
		}
	}

	// Circuit should be open now
	if cb.GetState() != "open" {
		t.Errorf("Expected state to be open after failures, got %s", cb.GetState())
	}

	// Test that requests are blocked when open
	err = cb.Execute(nil, func() error {
		return nil
	})
	if err == nil || err.Error() != "circuit breaker is open" {
		t.Errorf("Expected circuit breaker open error, got %v", err)
	}

	fmt.Println("Circuit breaker test passed!")
}

func TestGRPCErrorMapping(t *testing.T) {
	// Test nil error
	err := util.ConvertGRPCError(nil)
	if err != nil {
		t.Errorf("Expected nil for nil error, got %v", err)
	}

	// Test basic error conversion
	testErr := fmt.Errorf("test error")
	grpcErr := util.ConvertGRPCError(testErr)
	if grpcErr == nil {
		t.Error("Expected non-nil GRPC error")
	}
	if grpcErr.HTTPStatus != 500 {
		t.Errorf("Expected HTTP status 500, got %d", grpcErr.HTTPStatus)
	}

	fmt.Println("GRPC error mapping test passed!")
}
