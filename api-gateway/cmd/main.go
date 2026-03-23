package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"

	"github.com/flexsearch/api-gateway/internal/client"
	"github.com/flexsearch/api-gateway/internal/config"
	"github.com/flexsearch/api-gateway/internal/handler"
	"github.com/flexsearch/api-gateway/internal/middleware"
	"github.com/flexsearch/api-gateway/internal/util"
	"github.com/gin-gonic/gin"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"github.com/redis/go-redis/v9"
	"go.uber.org/zap"
)

func main() {
	cfg, err := config.Load()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	logger, err := util.NewLogger(cfg.Log.Level, cfg.Log.Format, cfg.Log.Output)
	if err != nil {
		log.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	if cfg.Server.Mode == "release" {
		gin.SetMode(gin.ReleaseMode)
	}

	tracingConfig := &middleware.TracingConfig{
		ServiceName: "api-gateway",
		Enabled:     true,
		SampleRate:  1.0,
	}
	shutdownTracing, err := middleware.InitTracing(tracingConfig, logger.Logger)
	if err != nil {
		logger.Warn("Failed to initialize tracing", zap.Error(err))
	}
	defer func() {
		if shutdownErr := shutdownTracing(context.Background()); shutdownErr != nil {
			logger.Error("Failed to shutdown tracing", zap.Error(shutdownErr))
		}
	}()

	metrics := util.NewMetrics("api_gateway")
	tracingMiddleware := middleware.NewTracingMiddleware(tracingConfig, logger.Logger)

	redisClient := redis.NewClient(&redis.Options{
		Addr:     fmt.Sprintf("%s:%d", cfg.Redis.Host, cfg.Redis.Port),
		Password: cfg.Redis.Password,
		DB:       cfg.Redis.DB,
	})
	defer redisClient.Close()

	ctx := context.Background()
	if pingErr := redisClient.Ping(ctx).Err(); pingErr != nil {
		logger.Error("Failed to connect to Redis", zap.Error(pingErr))
	} else {
		logger.Info("Connected to Redis successfully")
	}

	// Use enhanced rate limiter with burst capacity and tiers
	rateLimitConfig := util.DefaultRateLimitConfig()
	rateLimitConfig.Enabled = cfg.RateLimit.Enabled
	rateLimitConfig.DefaultLimit = cfg.RateLimit.DefaultLimit
	rateLimiter := util.NewRateLimiter(redisClient, rateLimitConfig)

	coordinatorClient, err := client.NewCircuitBreakerCoordinatorClient(&cfg.Coordinator)
	if err != nil {
		logger.Error("Failed to connect to coordinator", zap.Error(err))
	} else {
		logger.Info("Connected to coordinator successfully", zap.String("address", cfg.Coordinator.Address))
		defer coordinatorClient.Close()
	}

	jwtManager := util.NewJWTManager(cfg.JWT.Secret, cfg.JWT.Issuer, cfg.JWT.Expiration)

	router := gin.New()

	router.Use(gin.Recovery())
	router.Use(middleware.RequestIDMiddleware())
	router.Use(tracingMiddleware.Middleware())
	router.Use(middleware.RequestLoggingMiddleware(logger.Logger))
	router.Use(middleware.ErrorHandlerMiddleware(logger.Logger))
	router.Use(middleware.ResponseValidationMiddleware(logger.Logger, middleware.DefaultResponseValidationConfig()))

	router.GET("/metrics", gin.WrapH(promhttp.Handler()))

	if cfg.CORS.Enabled {
		router.Use(middleware.CORSMiddleware(middleware.CORSConfig{
			AllowOrigins:     cfg.CORS.AllowOrigins,
			AllowMethods:     cfg.CORS.AllowMethods,
			AllowHeaders:     cfg.CORS.AllowHeaders,
			AllowCredentials: cfg.CORS.AllowCredentials,
		}))
	}

	if cfg.RateLimit.Enabled {
		router.Use(middleware.RateLimitMiddleware(rateLimiter, middleware.RateLimitConfig{
			Enabled:       cfg.RateLimit.Enabled,
			DefaultLimit:  cfg.RateLimit.DefaultLimit,
			DefaultBurst:  20,
			DefaultWindow: "1m",
			ByUser:        cfg.RateLimit.ByUser,
			ByIP:          cfg.RateLimit.ByIP,
			TierHeader:    "X-RateLimit-Tier",
		}))
	}

	searchHandler := handler.NewSearchHandler(coordinatorClient.CoordinatorClient, metrics, logger.Logger)
	documentHandler := handler.NewDocumentHandler(coordinatorClient.CoordinatorClient, metrics, logger.Logger)
	indexHandler := handler.NewIndexHandler(coordinatorClient.CoordinatorClient, metrics, logger.Logger)
	healthHandler := handler.NewHealthHandler(coordinatorClient, cfg, logger.Logger)

	v1 := router.Group("/api/v1")
	{
		auth := v1.Group("")
		auth.Use(middleware.AuthMiddleware(jwtManager))
		{
			auth.POST("/search", searchHandler.Search)
			auth.GET("/search", searchHandler.SearchGet)

			auth.POST("/documents", documentHandler.Create)
			auth.GET("/documents/:index_id/:id", documentHandler.Get)
			auth.PUT("/documents/:index_id/:id", documentHandler.Update)
			auth.DELETE("/documents/:index_id/:id", documentHandler.Delete)
			auth.POST("/documents/batch", documentHandler.Batch)

			auth.POST("/indexes", indexHandler.Create)
			auth.GET("/indexes", indexHandler.List)
			auth.GET("/indexes/:id", indexHandler.Get)
			auth.DELETE("/indexes/:id", indexHandler.Delete)
			auth.POST("/indexes/:id/rebuild", indexHandler.Rebuild)
		}
	}

	router.GET("/health", healthHandler.Check)
	router.GET("/health/services", healthHandler.CheckServices)
	router.GET("/health/circuit-breakers", healthHandler.CheckCircuitBreakers)

	srv := &http.Server{
		Addr:           fmt.Sprintf(":%d", cfg.Server.Port),
		Handler:        router,
		ReadTimeout:    time.Duration(cfg.Server.ReadTimeout) * time.Second,
		WriteTimeout:   time.Duration(cfg.Server.WriteTimeout) * time.Second,
		MaxHeaderBytes: 1 << 20,
	}

	go func() {
		logger.Info("Starting server",
			zap.Int("port", cfg.Server.Port),
			zap.String("mode", cfg.Server.Mode))
		if err := srv.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			logger.Fatal("Failed to start server", zap.Error(err))
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)
	<-quit

	logger.Info("Shutting down server...")

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := srv.Shutdown(ctx); err != nil {
		logger.Error("Server forced to shutdown", zap.Error(err))
	}

	logger.Info("Server exited")
}
