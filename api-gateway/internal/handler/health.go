package handler

import (
	"context"
	"net/http"
	"time"

	"github.com/flexsearch/api-gateway/internal/client"
	"github.com/flexsearch/api-gateway/internal/config"
	"github.com/flexsearch/api-gateway/internal/middleware"
	pb "github.com/flexsearch/api-gateway/proto"
	"github.com/gin-gonic/gin"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/zap"
)

type HealthHandler struct {
	client               *client.CircuitBreakerCoordinatorClient
	config               *config.Config
	logger               *zap.Logger
	tracer               trace.Tracer
	circuitBreakerClient *client.CircuitBreakerCoordinatorClient
}

func NewHealthHandler(client *client.CircuitBreakerCoordinatorClient, cfg *config.Config, logger *zap.Logger) *HealthHandler {
	return &HealthHandler{
		client:               client,
		config:               cfg,
		logger:               logger,
		tracer:               otel.Tracer("health-handler"),
		circuitBreakerClient: client,
	}
}

func (h *HealthHandler) Check(c *gin.Context) {
	requestID := middleware.GetRequestID(c)

	c.JSON(http.StatusOK, gin.H{
		"status":     "healthy",
		"service":    "api-gateway",
		"timestamp":  time.Now().UTC().Format(time.RFC3339),
		"version":    "1.0.0",
		"request_id": requestID,
	})
}

func (h *HealthHandler) CheckServices(c *gin.Context) {
	ctx := c.Request.Context()
	ctx, span := h.tracer.Start(ctx, "HealthHandler.CheckServices")
	defer span.End()

	requestID := middleware.GetRequestID(c)
	span.SetAttributes(attribute.String("request_id", requestID))

	services := make(map[string]interface{})
	overallStatus := "healthy"

	coordinatorStatus := h.checkCoordinator(ctx)
	services["coordinator"] = coordinatorStatus
	if coordinatorStatus["status"] != "healthy" {
		overallStatus = "unhealthy"
	}

	// Add circuit breaker statistics
	if h.circuitBreakerClient != nil {
		circuitBreakerStats := h.circuitBreakerClient.GetCircuitBreakerStats()
		services["circuit_breakers"] = circuitBreakerStats
	}

	c.JSON(http.StatusOK, gin.H{
		"status":     overallStatus,
		"services":   services,
		"timestamp":  time.Now().UTC().Format(time.RFC3339),
		"request_id": requestID,
	})
}

func (h *HealthHandler) CheckCircuitBreakers(c *gin.Context) {
	if h.circuitBreakerClient == nil {
		c.JSON(http.StatusOK, gin.H{
			"error": "Circuit breaker client not available",
		})
		return
	}

	stats := h.circuitBreakerClient.GetCircuitBreakerStats()
	c.JSON(http.StatusOK, gin.H{
		"circuit_breakers": stats,
		"timestamp":        time.Now().UTC().Format(time.RFC3339),
	})
}

func (h *HealthHandler) checkCoordinator(ctx context.Context) map[string]interface{} {
	start := time.Now()
	ctx, span := h.tracer.Start(ctx, "HealthHandler.checkCoordinator")
	defer span.End()

	req := &pb.HealthCheckRequest{
		Service: "coordinator",
	}

	resp, err := h.client.HealthCheck(ctx, req)
	latency := time.Since(start)

	if err != nil {
		h.logger.Error("Coordinator health check failed", zap.Error(err))
		span.RecordError(err)
		return map[string]interface{}{
			"status":     "unhealthy",
			"latency_ms": latency.Milliseconds(),
			"address":    h.config.Coordinator.Address,
			"error":      err.Error(),
		}
	}

	span.SetAttributes(
		attribute.String("coordinator.status", resp.Status),
		attribute.Int64("coordinator.latency_ms", latency.Milliseconds()),
	)

	return map[string]interface{}{
		"status":         resp.Status,
		"version":        resp.Version,
		"uptime_seconds": resp.UptimeSeconds,
		"latency_ms":     latency.Milliseconds(),
		"address":        h.config.Coordinator.Address,
		"details":        resp.Details,
	}
}
