package util

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
	"sync"
	"time"
)

type Metrics struct {
	httpRequestsTotal    *prometheus.CounterVec
	httpRequestsDuration *prometheus.HistogramVec
	httpRequestsInFlight prometheus.Gauge
	searchLatency        *prometheus.HistogramVec
	documentOperations   *prometheus.CounterVec
	indexOperations      *prometheus.CounterVec
	errorCounter         *prometheus.CounterVec
	startTime            time.Time
	mu                   sync.RWMutex
}

func NewMetrics(namespace string) *Metrics {
	m := &Metrics{
		httpRequestsTotal: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "http_requests_total",
				Help:      "Total number of HTTP requests",
			},
			[]string{"method", "endpoint", "status"},
		),
		httpRequestsDuration: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace: namespace,
				Name:      "http_request_duration_seconds",
				Help:      "HTTP request duration in seconds",
				Buckets:   prometheus.DefBuckets,
			},
			[]string{"method", "endpoint"},
		),
		httpRequestsInFlight: promauto.NewGauge(
			prometheus.GaugeOpts{
				Namespace: namespace,
				Name:      "http_requests_in_flight",
				Help:      "Number of HTTP requests currently being processed",
			},
		),
		searchLatency: promauto.NewHistogramVec(
			prometheus.HistogramOpts{
				Namespace: namespace,
				Name:      "search_latency_seconds",
				Help:      "Search operation latency in seconds",
				Buckets:   prometheus.DefBuckets,
			},
			[]string{"index"},
		),
		documentOperations: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "document_operations_total",
				Help:      "Total number of document operations",
			},
			[]string{"operation", "status"},
		),
		indexOperations: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "index_operations_total",
				Help:      "Total number of index operations",
			},
			[]string{"operation", "status"},
		),
		errorCounter: promauto.NewCounterVec(
			prometheus.CounterOpts{
				Namespace: namespace,
				Name:      "errors_total",
				Help:      "Total number of errors",
			},
			[]string{"type", "location"},
		),
		startTime: time.Now(),
	}

	return m
}

func (m *Metrics) IncrementCounter(name string, labels []string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	switch name {
	case "http_requests_total":
		if len(labels) >= 3 {
			m.httpRequestsTotal.WithLabelValues(labels[0], labels[1], labels[2]).Inc()
		}
	case "document_operations_total":
		if len(labels) >= 2 {
			m.documentOperations.WithLabelValues(labels[0], labels[1]).Inc()
		}
	case "index_operations_total":
		if len(labels) >= 2 {
			m.indexOperations.WithLabelValues(labels[0], labels[1]).Inc()
		}
	case "errors_total":
		if len(labels) >= 2 {
			m.errorCounter.WithLabelValues(labels[0], labels[1]).Inc()
		}
	}
}

func (m *Metrics) RecordHistogram(name string, value float64, labels []string) {
	m.mu.Lock()
	defer m.mu.Unlock()

	switch name {
	case "http_request_duration_seconds":
		if len(labels) >= 2 {
			m.httpRequestsDuration.WithLabelValues(labels[0], labels[1]).Observe(value)
		}
	case "search_latency_seconds":
		if len(labels) >= 1 {
			m.searchLatency.WithLabelValues(labels[0]).Observe(value)
		}
	}
}

func (m *Metrics) IncrementHTTPRequest(method, endpoint, status string) {
	m.httpRequestsTotal.WithLabelValues(method, endpoint, status).Inc()
}

func (m *Metrics) RecordHTTPDuration(method, endpoint string, duration time.Duration) {
	m.httpRequestsDuration.WithLabelValues(method, endpoint).Observe(duration.Seconds())
}

func (m *Metrics) RecordSearchLatency(index string, duration time.Duration) {
	m.searchLatency.WithLabelValues(index).Observe(duration.Seconds())
}

func (m *Metrics) IncrementDocumentOperation(operation, status string) {
	m.documentOperations.WithLabelValues(operation, status).Inc()
}

func (m *Metrics) IncrementIndexOperation(operation, status string) {
	m.indexOperations.WithLabelValues(operation, status).Inc()
}

func (m *Metrics) IncrementError(errorType, location string) {
	m.errorCounter.WithLabelValues(errorType, location).Inc()
}

func (m *Metrics) IncrementInFlight() {
	m.httpRequestsInFlight.Inc()
}

func (m *Metrics) DecrementInFlight() {
	m.httpRequestsInFlight.Dec()
}

func (m *Metrics) GetUptime() time.Duration {
	return time.Since(m.startTime)
}
