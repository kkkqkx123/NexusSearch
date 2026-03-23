package metrics

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	redisConnectionsActive = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "redis_connections_active",
			Help: "Number of active Redis connections",
		},
		[]string{"service", "instance"},
	)

	redisConnectionsIdle = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "redis_connections_idle",
			Help: "Number of idle Redis connections",
		},
		[]string{"service", "instance"},
	)

	redisOperationsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "redis_operations_total",
			Help: "Total number of Redis operations",
		},
		[]string{"service", "operation", "status"},
	)

	redisOperationDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "redis_operation_duration_seconds",
			Help:    "Redis operation duration in seconds",
			Buckets: prometheus.DefBuckets,
		},
		[]string{"service", "operation"},
	)

	cacheHitsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "cache_hits_total",
			Help: "Total number of cache hits",
		},
		[]string{"service", "cache_type"},
	)

	cacheMissesTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "cache_misses_total",
			Help: "Total number of cache misses",
		},
		[]string{"service", "cache_type"},
	)

	cacheSizeBytes = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "cache_size_bytes",
			Help: "Current cache size in bytes",
		},
		[]string{"service", "cache_type"},
	)

	cacheEvictionsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "cache_evictions_total",
			Help: "Total number of cache evictions",
		},
		[]string{"service", "cache_type"},
	)
)

type RedisMetrics struct {
	serviceName string
	instance    string
}

func NewRedisMetrics(serviceName, instance string) *RedisMetrics {
	return &RedisMetrics{
		serviceName: serviceName,
		instance:    instance,
	}
}

func (rm *RedisMetrics) RecordConnection(active, idle int) {
	redisConnectionsActive.WithLabelValues(rm.serviceName, rm.instance).Set(float64(active))
	redisConnectionsIdle.WithLabelValues(rm.serviceName, rm.instance).Set(float64(idle))
}

func (rm *RedisMetrics) RecordOperation(operation, status string, duration float64) {
	redisOperationsTotal.WithLabelValues(rm.serviceName, operation, status).Inc()
	redisOperationDuration.WithLabelValues(rm.serviceName, operation).Observe(duration)
}

func (rm *RedisMetrics) RecordCacheHit(cacheType string) {
	cacheHitsTotal.WithLabelValues(rm.serviceName, cacheType).Inc()
}

func (rm *RedisMetrics) RecordCacheMiss(cacheType string) {
	cacheMissesTotal.WithLabelValues(rm.serviceName, cacheType).Inc()
}

func (rm *RedisMetrics) RecordCacheSize(cacheType string, size float64) {
	cacheSizeBytes.WithLabelValues(rm.serviceName, cacheType).Set(size)
}

func (rm *RedisMetrics) RecordCacheEviction(cacheType string) {
	cacheEvictionsTotal.WithLabelValues(rm.serviceName, cacheType).Inc()
}

func (rm *RedisMetrics) ServiceName() string {
	return rm.serviceName
}

func (rm *RedisMetrics) Instance() string {
	return rm.instance
}
