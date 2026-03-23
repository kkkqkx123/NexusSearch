package middleware

import (
	"context"
	"io"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
	"go.opentelemetry.io/otel"
	"go.opentelemetry.io/otel/attribute"
	"go.opentelemetry.io/otel/exporters/stdout/stdouttrace"
	"go.opentelemetry.io/otel/propagation"
	"go.opentelemetry.io/otel/sdk/resource"
	sdktrace "go.opentelemetry.io/otel/sdk/trace"
	semconv "go.opentelemetry.io/otel/semconv/v1.21.0"
	"go.opentelemetry.io/otel/trace"
	"go.uber.org/zap"
)

type TracingConfig struct {
	ServiceName string
	Enabled     bool
	SampleRate  float64
}

type TracingMiddleware struct {
	tracer trace.Tracer
	logger *zap.Logger
	config *TracingConfig
}

func InitTracing(config *TracingConfig, logger *zap.Logger) (func(context.Context) error, error) {
	if !config.Enabled {
		return func(ctx context.Context) error { return nil }, nil
	}

	exporter, err := stdouttrace.New(
		stdouttrace.WithPrettyPrint(),
		stdouttrace.WithWriter(io.Discard),
	)
	if err != nil {
		return nil, err
	}

	res, err := resource.New(context.Background(),
		resource.WithAttributes(
			semconv.ServiceName(config.ServiceName),
			semconv.ServiceVersion("1.0.0"),
		),
	)
	if err != nil {
		return nil, err
	}

	var sampler sdktrace.Sampler
	if config.SampleRate >= 1.0 {
		sampler = sdktrace.AlwaysSample()
	} else {
		sampler = sdktrace.TraceIDRatioBased(config.SampleRate)
	}

	tp := sdktrace.NewTracerProvider(
		sdktrace.WithBatcher(exporter),
		sdktrace.WithResource(res),
		sdktrace.WithSampler(sampler),
	)

	otel.SetTracerProvider(tp)
	otel.SetTextMapPropagator(propagation.NewCompositeTextMapPropagator(
		propagation.TraceContext{},
		propagation.Baggage{},
	))

	return func(ctx context.Context) error {
		return tp.Shutdown(ctx)
	}, nil
}

func NewTracingMiddleware(config *TracingConfig, logger *zap.Logger) *TracingMiddleware {
	tracer := otel.Tracer(config.ServiceName)
	return &TracingMiddleware{
		tracer: tracer,
		logger: logger,
		config: config,
	}
}

func (tm *TracingMiddleware) Middleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		if !tm.config.Enabled {
			c.Next()
			return
		}

		requestID := c.GetHeader("X-Request-ID")
		if requestID == "" {
			requestID = uuid.New().String()
		}

		ctx := context.Background()
		propagator := otel.GetTextMapPropagator()
		ctx = propagator.Extract(ctx, propagation.HeaderCarrier(c.Request.Header))

		ctx, span := tm.tracer.Start(ctx, c.Request.Method+" "+c.FullPath(),
			trace.WithSpanKind(trace.SpanKindServer),
			trace.WithAttributes(
				attribute.String("http.method", c.Request.Method),
				attribute.String("http.url", c.Request.URL.String()),
				attribute.String("http.target", c.Request.URL.Path),
				attribute.String("http.host", c.Request.Host),
				attribute.String("http.user_agent", c.Request.UserAgent()),
				attribute.String("http.request_id", requestID),
				attribute.String("http.client_ip", c.ClientIP()),
			),
		)
		defer span.End()

		c.Request = c.Request.WithContext(ctx)
		c.Set("request_id", requestID)
		c.Set("span", span)

		start := time.Now()
		c.Next()
		duration := time.Since(start)

		statusCode := c.Writer.Status()
		span.SetAttributes(
			attribute.Int("http.status_code", statusCode),
			attribute.Int64("http.response_content_length", int64(c.Writer.Size())),
			attribute.Float64("http.duration_ms", float64(duration.Milliseconds())),
		)

		if statusCode >= 400 {
			span.SetAttributes(attribute.Bool("error", true))
		}

		c.Header("X-Request-ID", requestID)
	}
}

func GetSpanFromContext(ctx context.Context) trace.Span {
	return trace.SpanFromContext(ctx)
}

func GetTracerFromContext(serviceName string) trace.Tracer {
	return otel.Tracer(serviceName)
}

func AddSpanAttributes(ctx context.Context, attrs ...attribute.KeyValue) {
	span := trace.SpanFromContext(ctx)
	span.SetAttributes(attrs...)
}

func AddSpanEvent(ctx context.Context, name string, attrs ...attribute.KeyValue) {
	span := trace.SpanFromContext(ctx)
	span.AddEvent(name, trace.WithAttributes(attrs...))
}

func RecordError(ctx context.Context, err error) {
	span := trace.SpanFromContext(ctx)
	span.RecordError(err)
}

func StartSpan(ctx context.Context, name string, serviceName string, opts ...trace.SpanStartOption) (context.Context, trace.Span) {
	tracer := otel.Tracer(serviceName)
	return tracer.Start(ctx, name, opts...)
}

type TraceContext struct {
	TraceID string `json:"trace_id"`
	SpanID  string `json:"span_id"`
}

func GetTraceContext(ctx context.Context) *TraceContext {
	span := trace.SpanFromContext(ctx)
	if span.SpanContext().HasTraceID() {
		return &TraceContext{
			TraceID: span.SpanContext().TraceID().String(),
			SpanID:  span.SpanContext().SpanID().String(),
		}
	}
	return nil
}

func NopTracer() trace.Tracer {
	return otel.Tracer("nop")
}
