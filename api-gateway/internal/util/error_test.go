package util

import (
	"errors"
	"testing"
)

func TestAppError_Error(t *testing.T) {
	err := &AppError{
		Code:    400,
		Message: "Bad request",
		Details: "Invalid parameter",
	}

	expected := "Bad request"
	if err.Error() != expected {
		t.Errorf("Expected error message '%s', got '%s'", expected, err.Error())
	}
}

func TestAppError_NewAppError(t *testing.T) {
	err := NewAppError(404, "Not found", "Resource does not exist")

	if err.Code != 404 {
		t.Errorf("Expected Code 404, got %d", err.Code)
	}

	if err.Message != "Not found" {
		t.Errorf("Expected Message 'Not found', got '%s'", err.Message)
	}

	if err.Details != "Resource does not exist" {
		t.Errorf("Expected Details 'Resource does not exist', got '%s'", err.Details)
	}
}

func TestWrapError_AppError(t *testing.T) {
	appErr := &AppError{
		Code:    401,
		Message: "Unauthorized",
		Details: "Invalid token",
	}

	wrappedErr := WrapError(appErr, "wrapped error")

	if wrappedErr.Code != 401 {
		t.Errorf("Expected Code 401, got %d", wrappedErr.Code)
	}

	if wrappedErr.Message != "Unauthorized" {
		t.Errorf("Expected Message 'Unauthorized', got '%s'", wrappedErr.Message)
	}
}

func TestWrapError_StandardError(t *testing.T) {
	originalErr := errors.New("database connection failed")

	wrappedErr := WrapError(originalErr, "database error occurred")

	if wrappedErr.Code != 500 {
		t.Errorf("Expected Code 500, got %d", wrappedErr.Code)
	}

	if wrappedErr.Message != "database error occurred" {
		t.Errorf("Expected Message 'database error occurred', got '%s'", wrappedErr.Message)
	}

	if wrappedErr.Details != "database connection failed" {
		t.Errorf("Expected Details 'database connection failed', got '%s'", wrappedErr.Details)
	}
}

func TestWrapError_NilError(t *testing.T) {
	wrappedErr := WrapError(nil, "nil error")

	if wrappedErr != nil {
		t.Error("Expected nil for nil error")
	}
}

func TestFormatError(t *testing.T) {
	testCases := []struct {
		name     string
		err      error
		expected string
	}{
		{
			name:     "nil error",
			err:      nil,
			expected: "",
		},
		{
			name:     "standard error",
			err:      errors.New("test error"),
			expected: "test error",
		},
		{
			name:     "AppError",
			err:      &AppError{Code: 400, Message: "Bad request"},
			expected: "Bad request",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			result := FormatError(tc.err)
			if result != tc.expected {
				t.Errorf("Expected '%s', got '%s'", tc.expected, result)
			}
		})
	}
}

func TestPredefinedErrors(t *testing.T) {
	testCases := []struct {
		err         *AppError
		expected    int
		expectedMsg string
	}{
		{ErrUnauthorized, 401, "Unauthorized"},
		{ErrForbidden, 403, "Forbidden"},
		{ErrNotFound, 404, "Not found"},
		{ErrRateLimitExceeded, 429, "Rate limit exceeded"},
		{ErrInternalServer, 500, "Internal server error"},
		{ErrBadRequest, 400, "Bad request"},
		{ErrServiceUnavailable, 503, "Service unavailable"},
	}

	for _, tc := range testCases {
		t.Run(tc.expectedMsg, func(t *testing.T) {
			if tc.err.Code != tc.expected {
				t.Errorf("Expected Code %d, got %d", tc.expected, tc.err.Code)
			}

			if tc.err.Message != tc.expectedMsg {
				t.Errorf("Expected Message '%s', got '%s'", tc.expectedMsg, tc.err.Message)
			}

			if tc.err.Details != "" {
				t.Errorf("Expected empty Details, got '%s'", tc.err.Details)
			}
		})
	}
}
