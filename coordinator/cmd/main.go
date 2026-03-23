package main

import (
	"context"
	"fmt"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/flexsearch/coordinator/internal/cache"
	"github.com/flexsearch/coordinator/internal/config"
	"github.com/flexsearch/coordinator/internal/engine"
	"github.com/flexsearch/coordinator/internal/merger"
	"github.com/flexsearch/coordinator/internal/router"
	coordinatorServer "github.com/flexsearch/coordinator/internal/server"
	"github.com/flexsearch/coordinator/internal/service"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/prometheus/client_golang/prometheus/promhttp"
	"google.golang.org/grpc"
	"google.golang.org/grpc/health"
	healthpb "google.golang.org/grpc/health/grpc_health_v1"
	"google.golang.org/grpc/reflection"
)

const (
	serviceName = "coordinator"
)

func main() {
	cfg, err := config.Load("configs/config.yaml")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to load config: %v\n", err)
		os.Exit(1)
	}

	logger, err := util.NewLogger(cfg.Logging.Level, cfg.Logging.Format, cfg.Logging.Output)
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to initialize logger: %v\n", err)
		os.Exit(1)
	}
	defer logger.Sync()

	metrics := util.NewMetrics(serviceName)

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	redisCache, err := cache.NewRedisCache(&cache.CacheConfig{
		Enabled:    cfg.Cache.Enabled,
		Host:       cfg.Redis.Host,
		Port:       cfg.Redis.Port,
		Password:   cfg.Redis.Password,
		DB:         cfg.Redis.DB,
		PoolSize:   cfg.Redis.PoolSize,
		DefaultTTL: cfg.Cache.DefaultTTL,
	}, logger)
	if err != nil {
		logger.Warnf("Redis cache initialization failed: %v", err)
	}

	engines := initializeEngines(cfg, logger)

	r := router.NewRouter(logger)
	optimizer := router.NewOptimizer(logger)

	mergerConfig := &merger.MergerConfig{
		Strategy: "rrf",
		RRFK:     60,
		TopK:     100,
	}
	resultMerger := merger.NewMerger("rrf", mergerConfig, logger)

	searchService := service.NewSearchService(&service.SearchServiceConfig{
		Config:    cfg,
		Logger:    logger,
		Cache:     redisCache,
		Router:    r,
		Optimizer: optimizer,
		Merger:    resultMerger,
		Engines:   engines,
		Metrics:   metrics,
	})

	grpcServer := setupGRPCServer(cfg, logger, searchService)
	metricsServer := setupMetricsServer(cfg, metrics)

	if cfg.Metrics.Enabled {
		go func() {
			addr := cfg.GetMetricsAddress()
			logger.Infof("Starting metrics server on %s", addr)
			if err := metricsServer.ListenAndServe(); err != nil && err != http.ErrServerClosed {
				logger.Errorf("Metrics server error: %v", err)
			}
		}()
	}

	go func() {
		addr := cfg.GetGRPCAddress()
		logger.Infof("Starting gRPC server on %s", addr)
		listener, err := net.Listen("tcp", addr)
		if err != nil {
			logger.Fatalf("Failed to listen: %v", err)
		}
		if err := grpcServer.Serve(listener); err != nil {
			logger.Fatalf("Failed to serve: %v", err)
		}
	}()

	logger.Infof("%s service started successfully", serviceName)

	waitForShutdown(ctx, cancel, cfg, grpcServer, metricsServer, logger)
}

func initializeEngines(cfg *config.Config, logger *util.Logger) map[string]engine.EngineClient {
	engines := make(map[string]engine.EngineClient)

	if cfg.Engines.FlexSearch.Enabled {
		flexClient := engine.NewFlexSearchClient(&engine.ClientConfig{
			Host:       cfg.Engines.FlexSearch.Host,
			Port:       cfg.Engines.FlexSearch.Port,
			Timeout:    cfg.Engines.FlexSearch.Timeout,
			MaxRetries: cfg.Engines.FlexSearch.MaxRetries,
			PoolSize:   cfg.Engines.FlexSearch.PoolSize,
		}, logger)
		if err := flexClient.Connect(context.Background()); err != nil {
			logger.Warnf("Failed to connect to FlexSearch: %v", err)
		} else {
			engines["flexsearch"] = flexClient
		}
	}

	if cfg.Engines.BM25.Enabled {
		bm25Client := engine.NewBM25Client(&engine.ClientConfig{
			Host:       cfg.Engines.BM25.Host,
			Port:       cfg.Engines.BM25.Port,
			Timeout:    cfg.Engines.BM25.Timeout,
			MaxRetries: cfg.Engines.BM25.MaxRetries,
			PoolSize:   cfg.Engines.BM25.PoolSize,
		}, &engine.BM25EngineConfig{
			K1:        cfg.Engines.BM25.K1,
			B:         cfg.Engines.BM25.B,
			MinLength: 2,
			MaxLength: 100,
		}, logger)
		if err := bm25Client.Connect(context.Background()); err != nil {
			logger.Warnf("Failed to connect to BM25: %v", err)
		} else {
			engines["bm25"] = bm25Client
		}
	}

	if cfg.Engines.Vector.Enabled {
		vectorClient, err := engine.NewVectorClient(&engine.ClientConfig{
			Host:       cfg.Engines.Vector.Host,
			Port:       cfg.Engines.Vector.Port,
			Timeout:    cfg.Engines.Vector.Timeout,
			MaxRetries: cfg.Engines.Vector.MaxRetries,
			PoolSize:   cfg.Engines.Vector.PoolSize,
		}, &engine.VectorEngineConfig{
			Model:     cfg.Engines.Vector.Model,
			Dimension: cfg.Engines.Vector.Dimension,
			Threshold: 0.7,
			TopK:      10,
			Hybrid:    false,
			Alpha:     0.5,
		}, logger)
		if err != nil {
			logger.Errorf("Failed to create Vector client: %v", err)
		} else if err := vectorClient.Connect(context.Background()); err != nil {
			logger.Warnf("Failed to connect to Vector: %v", err)
		} else {
			engines["vector"] = vectorClient
		}
	}

	logger.Infof("Initialized %d engines", len(engines))
	return engines
}

func setupGRPCServer(cfg *config.Config, logger *util.Logger, searchService *service.SearchService) *grpc.Server {
	opts := []grpc.ServerOption{
		grpc.MaxRecvMsgSize(cfg.GRPC.MaxRecvMsgSize),
		grpc.MaxSendMsgSize(cfg.GRPC.MaxSendMsgSize),
	}

	server := grpc.NewServer(opts...)

	coordinatorServer.NewCoordinatorServer(logger, searchService)

	healthServer := health.NewServer()
	healthServer.SetServingStatus("", healthpb.HealthCheckResponse_SERVING)
	healthServer.SetServingStatus(serviceName, healthpb.HealthCheckResponse_SERVING)

	reflection.Register(server)

	logger.Info("gRPC server configured")
	return server
}

func setupMetricsServer(cfg *config.Config, metrics *util.Metrics) *http.Server {
	mux := http.NewServeMux()
	mux.Handle(cfg.Metrics.Path, promhttp.Handler())

	return &http.Server{
		Addr:    cfg.GetMetricsAddress(),
		Handler: mux,
	}
}

func waitForShutdown(ctx context.Context, cancel context.CancelFunc, cfg *config.Config, grpcServer *grpc.Server, metricsServer *http.Server, logger *util.Logger) {
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigChan
	logger.Infof("Received signal: %v", sig)

	cancel()

	shutdownCtx, shutdownCancel := context.WithTimeout(context.Background(), cfg.Server.ShutdownTimeout)
	defer shutdownCancel()

	if metricsServer != nil && cfg.Metrics.Enabled {
		logger.Info("Shutting down metrics server...")
		if err := metricsServer.Shutdown(shutdownCtx); err != nil {
			logger.Errorf("Metrics server shutdown error: %v", err)
		}
	}

	logger.Info("Shutting down gRPC server...")
	stopped := make(chan struct{})
	go func() {
		grpcServer.GracefulStop()
		close(stopped)
	}()

	select {
	case <-stopped:
		logger.Info("gRPC server stopped gracefully")
	case <-shutdownCtx.Done():
		logger.Warn("gRPC server shutdown timeout, forcing stop...")
		grpcServer.Stop()
	}

	logger.Infof("%s service stopped", serviceName)
}
