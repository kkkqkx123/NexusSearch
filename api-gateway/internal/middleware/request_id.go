package middleware

import (
	"github.com/gin-gonic/gin"
	"github.com/google/uuid"
)

// RequestIDMiddleware generates and attaches a request ID to the context and response headers
func RequestIDMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		// Check if request ID already exists in headers
		requestID := c.GetHeader("X-Request-ID")
		if requestID == "" {
			// Generate new UUID if not provided
			requestID = uuid.New().String()
		}

		// Set request ID in context for use in handlers
		c.Set("request_id", requestID)

		// Add request ID to response headers
		c.Header("X-Request-ID", requestID)

		// Also add to request headers for downstream services
		c.Request.Header.Set("X-Request-ID", requestID)

		c.Next()
	}
}

// GetRequestID retrieves the request ID from the context
func GetRequestID(c *gin.Context) string {
	if requestID, exists := c.Get("request_id"); exists {
		if id, ok := requestID.(string); ok {
			return id
		}
	}
	return ""
}
