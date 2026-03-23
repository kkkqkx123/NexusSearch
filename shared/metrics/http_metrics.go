package metrics

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	httpRequestsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "http_requests_total",
			Help: "Total number of HTTP requests",
		},
		[]string{"service", "method", "path", "status"},
	)

	httpRequestDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "http_request_duration_seconds",
			Help:    "HTTP request duration in seconds",
			Buckets: []float64{0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10},
		},
		[]string{"service", "method", "path"},
	)

	httpRequestSize = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "http_request_size_bytes",
			Help:    "HTTP request size in bytes",
			Buckets: []float64{100, 1000, 10000, 100000, 1000000, 10000000},
		},
		[]string{"service", "method", "path"},
	)

	httpResponseSize = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "http_response_size_bytes",
			Help:    "HTTP response size in bytes",
			Buckets: []float64{100, 1000, 10000, 100000, 1000000, 10000000},
		},
		[]string{"service", "method", "path"},
	)

	httpInFlightRequests = promauto.NewGaugeVec(
		prometheus.GaugeOpts{
			Name: "http_in_flight_requests",
			Help: "Number of in-flight HTTP requests",
		},
		[]string{"service", "method", "path"},
	)
)

type HTTPMetrics struct {
	serviceName string
}

func NewHTTPMetrics(serviceName string) *HTTPMetrics {
	return &HTTPMetrics{
		serviceName: serviceName,
	}
}

func (hm *HTTPMetrics) RecordRequest(method, path, status string) {
	httpRequestsTotal.WithLabelValues(hm.serviceName, method, path, status).Inc()
}

func (hm *HTTPMetrics) RecordDuration(method, path string, duration float64) {
	httpRequestDuration.WithLabelValues(hm.serviceName, method, path).Observe(duration)
}

func (hm *HTTPMetrics) RecordRequestSize(method, path string, size float64) {
	httpRequestSize.WithLabelValues(hm.serviceName, method, path).Observe(size)
}

func (hm *HTTPMetrics) RecordResponseSize(method, path string, size float64) {
	httpResponseSize.WithLabelValues(hm.serviceName, method, path).Observe(size)
}

func (hm *HTTPMetrics) IncInFlight(method, path string) {
	httpInFlightRequests.WithLabelValues(hm.serviceName, method, path).Inc()
}

func (hm *HTTPMetrics) DecInFlight(method, path string) {
	httpInFlightRequests.WithLabelValues(hm.serviceName, method, path).Dec()
}

func (hm *HTTPMetrics) ServiceName() string {
	return hm.serviceName
}
