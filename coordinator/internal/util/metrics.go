package util

import (
	"sync"
	"time"

	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

type Metrics struct {
	grpcRequestsTotal    *prometheus.CounterVec
	grpcRequestsDuration *prometheus.HistogramVec
	grpcRequestsInFlight prometheus.Gauge
	queryLatency         *prometheus.HistogramVec
	engineLatency        *prometheus.HistogramVec
	mergerLatency        *prometheus.HistogramVec
	cacheHits            prometheus.Counter
	cacheMisses          prometheus.Counter
	searchRequestsTotal   *prometheus.CounterVec
	searchResultsTotal    *prometheus.CounterVec
	searchErrorsTotal     *prometheus.CounterVec
	startTime            time.Time
	mu                   sync.RWMutex
}

func NewMetrics(namespace string) *Metrics {
	m := &Metrics{
		grpcRequestsTotal: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "grpc_requests_total",
				Help:      "Total number of gRPC requests",
			},
			[]string{"method", "status"},
		),
		grpcRequestsDuration: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace: namespace,
				Name:      "grpc_request_duration_seconds",
				Help:      "gRPC request duration in seconds",
				Buckets:   prometheus.DefBuckets,
			},
			[]string{"method"},
		),
		grpcRequestsInFlight: promauto.NewGauge(
			prometheus.GaugeOpts{
				Namespace: namespace,
				Name:      "grpc_requests_in_flight",
				Help:      "Number of gRPC requests currently being processed",
			},
		),
		queryLatency: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace: namespace,
				Name:      "query_latency_seconds",
				Help:      "Query operation latency in seconds",
				Buckets:   prometheus.DefBuckets,
			},
			[]string{"query_type"},
		),
		engineLatency: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace: namespace,
				Name:      "engine_latency_seconds",
				Help:      "Search engine latency in seconds",
				Buckets:   prometheus.DefBuckets,
			},
			[]string{"engine", "operation"},
		),
		mergerLatency: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace: namespace,
				Name:      "merger_latency_seconds",
				Help:      "Result merger latency in seconds",
				Buckets:   prometheus.DefBuckets,
			},
			[]string{"strategy"},
		),
		cacheHits: promauto.NewCounter(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "cache_hits_total",
				Help:      "Total number of cache hits",
			},
		),
		cacheMisses: promauto.NewCounter(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "cache_misses_total",
				Help:      "Total number of cache misses",
			},
		),
		searchRequestsTotal: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "search_requests_total",
				Help:      "Total number of search requests",
			},
			[]string{"engine"},
		),
		searchResultsTotal: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "search_results_total",
				Help:      "Total number of search results",
			},
			[]string{"engine"},
		),
		searchErrorsTotal: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "search_errors_total",
				Help:      "Total number of search errors",
			},
			[]string{"engine", "error_type"},
		),
		startTime: time.Now(),
	}

	return m
}

func (m *Metrics) IncrementGRPCRequest(method, status string) {
	m.grpcRequestsTotal.WithLabelValues(method, status).Inc()
}

func (m *Metrics) RecordGRPCDuration(method string, duration time.Duration) {
	m.grpcRequestsDuration.WithLabelValues(method).Observe(duration.Seconds())
}

func (m *Metrics) IncrementInFlight() {
	m.grpcRequestsInFlight.Inc()
}

func (m *Metrics) DecrementInFlight() {
	m.grpcRequestsInFlight.Dec()
}

func (m *Metrics) RecordQueryLatency(queryType string, duration time.Duration) {
	m.queryLatency.WithLabelValues(queryType).Observe(duration.Seconds())
}

func (m *Metrics) RecordEngineLatency(engine, operation string, duration time.Duration) {
	m.engineLatency.WithLabelValues(engine, operation).Observe(duration.Seconds())
}

func (m *Metrics) RecordCacheHit() {
	m.cacheHits.Inc()
}

func (m *Metrics) RecordCacheMiss() {
	m.cacheMisses.Inc()
}

func (m *Metrics) RecordMergerLatency(strategy string, duration time.Duration) {
	m.mergerLatency.WithLabelValues(strategy).Observe(duration.Seconds())
}

func (m *Metrics) GetUptime() time.Duration {
	return time.Since(m.startTime)
}

func (m *Metrics) RecordSearchDuration(duration float64) {
	m.queryLatency.WithLabelValues("search").Observe(duration / 1000.0)
}

func (m *Metrics) RecordSearchResults(count int) {
	m.searchResultsTotal.WithLabelValues("coordinator").Add(float64(count))
}

func (m *Metrics) RecordSearchRequest(engine string) {
	m.searchRequestsTotal.WithLabelValues(engine).Inc()
}

func (m *Metrics) RecordSearchError(engine, errorType string) {
	m.searchErrorsTotal.WithLabelValues(engine, errorType).Inc()
}
