package metrics

import (
	"github.com/prometheus/client_golang/prometheus"
	"github.com/prometheus/client_golang/prometheus/promauto"
)

var (
	searchRequestsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "search_requests_total",
			Help: "Total number of search requests",
		},
		[]string{"service", "engine"},
	)

	searchDuration = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "search_duration_seconds",
			Help:    "Search duration in seconds",
			Buckets: []float64{0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10},
		},
		[]string{"service", "engine"},
	)

	searchResultsCount = promauto.NewHistogramVec(
		prometheus.HistogramOpts{
			Name:    "search_results_count",
			Help:    "Number of search results returned",
			Buckets: []float64{0, 1, 5, 10, 20, 50, 100, 200, 500, 1000},
		},
		[]string{"service", "engine"},
	)

	searchErrorsTotal = promauto.NewCounterVec(
		prometheus.CounterOpts{
			Name: "search_errors_total",
			Help: "Total number of search errors",
		},
		[]string{"service", "engine", "error_type"},
	)
)

type SearchMetrics struct {
	serviceName string
}

func NewSearchMetrics(serviceName string) *SearchMetrics {
	return &SearchMetrics{
		serviceName: serviceName,
	}
}

func (sm *SearchMetrics) RecordRequest(engine string) {
	searchRequestsTotal.WithLabelValues(sm.serviceName, engine).Inc()
}

func (sm *SearchMetrics) RecordDuration(engine string, duration float64) {
	searchDuration.WithLabelValues(sm.serviceName, engine).Observe(duration)
}

func (sm *SearchMetrics) RecordResults(engine string, count float64) {
	searchResultsCount.WithLabelValues(sm.serviceName, engine).Observe(count)
}

func (sm *SearchMetrics) RecordError(engine, errorType string) {
	searchErrorsTotal.WithLabelValues(sm.serviceName, engine, errorType).Inc()
}

func (sm *SearchMetrics) ServiceName() string {
	return sm.serviceName
}
