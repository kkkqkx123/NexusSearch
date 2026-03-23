package middleware

import (
	"time"

	"github.com/flexsearch/api-gateway/internal/util"
	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

type LoggingMiddleware struct {
	logger *util.Logger
}

func NewLoggingMiddleware(logger *util.Logger) *LoggingMiddleware {
	return &LoggingMiddleware{logger: logger}
}

func (lm *LoggingMiddleware) Middleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()
		path := c.Request.URL.Path
		query := c.Request.URL.RawQuery

		requestID := GetRequestID(c)

		lm.logger.Infow("HTTP request started",
			"method", c.Request.Method,
			"path", path,
			"query", query,
			"ip", c.ClientIP(),
			"request_id", requestID,
		)

		c.Next()

		latency := time.Since(start)
		status := c.Writer.Status()
		method := c.Request.Method
		clientIP := c.ClientIP()

		if query != "" {
			path = path + "?" + query
		}

		if status >= 400 {
			lm.logger.Errorw("HTTP request completed with error",
				"method", method,
				"path", path,
				"status_code", status,
				"latency_ms", latency.Milliseconds(),
				"ip", clientIP,
				"request_id", requestID,
				"response_size", c.Writer.Size(),
			)
		} else {
			lm.logger.Infow("HTTP request completed",
				"method", method,
				"path", path,
				"status_code", status,
				"latency_ms", latency.Milliseconds(),
				"ip", clientIP,
				"request_id", requestID,
				"response_size", c.Writer.Size(),
			)
		}
	}
}

func RequestLoggingMiddleware(logger *zap.Logger) gin.HandlerFunc {
	return func(c *gin.Context) {
		start := time.Now()
		path := c.Request.URL.Path
		query := c.Request.URL.RawQuery

		requestID := GetRequestID(c)

		logger.Info("HTTP request started",
			zap.String("method", c.Request.Method),
			zap.String("path", path),
			zap.String("query", query),
			zap.String("ip", c.ClientIP()),
			zap.String("request_id", requestID),
		)

		c.Next()

		latency := time.Since(start)
		status := c.Writer.Status()

		if status >= 400 {
			logger.Error("HTTP request completed with error",
				zap.String("method", c.Request.Method),
				zap.String("path", path),
				zap.Int("status_code", status),
				zap.Duration("latency", latency),
				zap.String("ip", c.ClientIP()),
				zap.String("request_id", requestID),
				zap.Int("response_size", c.Writer.Size()),
			)
		} else {
			logger.Info("HTTP request completed",
				zap.String("method", c.Request.Method),
				zap.String("path", path),
				zap.Int("status_code", status),
				zap.Duration("latency", latency),
				zap.String("ip", c.ClientIP()),
				zap.String("request_id", requestID),
				zap.Int("response_size", c.Writer.Size()),
			)
		}
	}
}

type ErrorHandlerConfig struct {
	IncludeStack bool
}

func ErrorHandlerMiddleware(logger *zap.Logger, config ...ErrorHandlerConfig) gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Next()

		if len(c.Errors) > 0 {
			err := c.Errors.Last()
			logger.Error("Request error",
				zap.String("error", err.Error()),
				zap.String("path", c.Request.URL.Path),
				zap.String("method", c.Request.Method),
				zap.Int("status", c.Writer.Status()),
			)

			c.JSON(-1, gin.H{
				"error": gin.H{
					"code":    "INTERNAL_ERROR",
					"message": err.Error(),
				},
			})
		}
	}
}
