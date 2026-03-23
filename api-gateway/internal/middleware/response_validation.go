package middleware

import (
	"fmt"
	"net/http"
	"reflect"
	"strconv"

	"github.com/gin-gonic/gin"
	"go.uber.org/zap"
)

// ValidatableResponse interface for responses that can be validated
type ValidatableResponse interface {
	Validate() error
}

// ResponseValidationConfig holds configuration for response validation
type ResponseValidationConfig struct {
	Enabled         bool
	ValidateOnError bool  // Whether to validate responses even when status >= 400
	MaxResponseSize int64 // Maximum response size in bytes
}

// DefaultResponseValidationConfig returns default configuration
func DefaultResponseValidationConfig() ResponseValidationConfig {
	return ResponseValidationConfig{
		Enabled:         true,
		ValidateOnError: false,
		MaxResponseSize: 10 * 1024 * 1024, // 10MB
	}
}

// ResponseValidationMiddleware validates HTTP responses
func ResponseValidationMiddleware(logger *zap.Logger, config ResponseValidationConfig) gin.HandlerFunc {
	return func(c *gin.Context) {
		if !config.Enabled {
			c.Next()
			return
		}

		// Create a custom response writer to capture the response
		writer := &responseCaptureWriter{
			ResponseWriter: c.Writer,
			body:           make([]byte, 0),
			maxSize:        config.MaxResponseSize,
		}
		c.Writer = writer

		c.Next()

		// Only validate successful responses unless configured otherwise
		if c.Writer.Status() >= 400 && !config.ValidateOnError {
			return
		}

		// Validate response if it implements ValidatableResponse
		if response, exists := c.Get("response"); exists {
			if validatable, ok := response.(ValidatableResponse); ok {
				if err := validatable.Validate(); err != nil {
					logger.Error("Response validation failed",
						zap.String("path", c.Request.URL.Path),
						zap.Int("status", c.Writer.Status()),
						zap.Error(err),
					)

					// Return validation error
					c.JSON(http.StatusInternalServerError, gin.H{
						"error":   "Response validation failed",
						"details": err.Error(),
					})
					return
				}
			}
		}

		// Validate response body size
		if int64(len(writer.body)) > config.MaxResponseSize {
			logger.Error("Response too large",
				zap.String("path", c.Request.URL.Path),
				zap.Int("status", c.Writer.Status()),
				zap.Int("response_size", len(writer.body)),
				zap.Int64("max_size", config.MaxResponseSize),
			)
		}
	}
}

// responseCaptureWriter captures the response body for validation
type responseCaptureWriter struct {
	gin.ResponseWriter
	body    []byte
	maxSize int64
}

func (w *responseCaptureWriter) Write(data []byte) (int, error) {
	// Check if adding this data would exceed max size
	if int64(len(w.body)+len(data)) > w.maxSize {
		return 0, fmt.Errorf("response size exceeds maximum allowed size of %d bytes", w.maxSize)
	}

	// Capture the data
	w.body = append(w.body, data...)

	// Write to the underlying ResponseWriter
	return w.ResponseWriter.Write(data)
}

// ValidationError represents a validation error with field information
type ValidationError struct {
	Field   string `json:"field"`
	Message string `json:"message"`
	Code    string `json:"code"`
}

// ValidateStruct validates a struct using reflection and basic rules
func ValidateStruct(data interface{}) []ValidationError {
	var errors []ValidationError

	v := reflect.ValueOf(data)
	if v.Kind() == reflect.Ptr {
		v = v.Elem()
	}

	if v.Kind() != reflect.Struct {
		return errors
	}

	t := v.Type()

	for i := 0; i < v.NumField(); i++ {
		field := v.Field(i)
		fieldType := t.Field(i)

		// Skip unexported fields
		if !field.CanInterface() {
			continue
		}

		// Basic validation rules
		switch field.Kind() {
		case reflect.String:
			str := field.String()
			if str == "" && isRequired(fieldType) {
				errors = append(errors, ValidationError{
					Field:   fieldType.Name,
					Message: "Field is required",
					Code:    "REQUIRED",
				})
			}
			if len(str) > getMaxLength(fieldType) {
				errors = append(errors, ValidationError{
					Field:   fieldType.Name,
					Message: "Field exceeds maximum length",
					Code:    "MAX_LENGTH",
				})
			}
		case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
			if field.Int() < getMinValue(fieldType) {
				errors = append(errors, ValidationError{
					Field:   fieldType.Name,
					Message: "Value is below minimum",
					Code:    "MIN_VALUE",
				})
			}
			if field.Int() > getMaxValue(fieldType) {
				errors = append(errors, ValidationError{
					Field:   fieldType.Name,
					Message: "Value exceeds maximum",
					Code:    "MAX_VALUE",
				})
			}
		}
	}

	return errors
}

// Helper functions to extract validation rules from struct tags
func isRequired(field reflect.StructField) bool {
	return field.Tag.Get("validate") == "required" || field.Tag.Get("binding") == "required"
}

func getMaxLength(field reflect.StructField) int {
	if max := field.Tag.Get("max"); max != "" {
		if length, err := strconv.Atoi(max); err == nil {
			return length
		}
	}
	return 1000 // Default max length
}

func getMinValue(field reflect.StructField) int64 {
	if min := field.Tag.Get("min"); min != "" {
		if value, err := strconv.ParseInt(min, 10, 64); err == nil {
			return value
		}
	}
	return 0 // Default min value
}

func getMaxValue(field reflect.StructField) int64 {
	if max := field.Tag.Get("max"); max != "" {
		if value, err := strconv.ParseInt(max, 10, 64); err == nil {
			return value
		}
	}
	return 1000000 // Default max value
}
