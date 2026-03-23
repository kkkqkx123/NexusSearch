package util

import "fmt"

type AppError struct {
	Code    int    `json:"code"`
	Message string `json:"message"`
	Details string `json:"details,omitempty"`
}

func (e *AppError) Error() string {
	return e.Message
}

func NewAppError(code int, message, details string) *AppError {
	return &AppError{
		Code:    code,
		Message: message,
		Details: details,
	}
}

var (
	ErrUnauthorized       = &AppError{Code: 401, Message: "Unauthorized"}
	ErrForbidden          = &AppError{Code: 403, Message: "Forbidden"}
	ErrNotFound           = &AppError{Code: 404, Message: "Not found"}
	ErrRateLimitExceeded  = &AppError{Code: 429, Message: "Rate limit exceeded"}
	ErrInternalServer     = &AppError{Code: 500, Message: "Internal server error"}
	ErrBadRequest         = &AppError{Code: 400, Message: "Bad request"}
	ErrServiceUnavailable = &AppError{Code: 503, Message: "Service unavailable"}
	ErrEngineTimeout      = &AppError{Code: 504, Message: "Engine timeout"}
	ErrEngineUnavailable  = &AppError{Code: 503, Message: "Engine unavailable"}
	ErrQueryInvalid       = &AppError{Code: 400, Message: "Invalid query"}
	ErrCacheError         = &AppError{Code: 500, Message: "Cache error"}
	ErrMergerError        = &AppError{Code: 500, Message: "Merger error"}
)

func WrapError(err error, message string) *AppError {
	if err == nil {
		return nil
	}
	if appErr, ok := err.(*AppError); ok {
		return appErr
	}
	return &AppError{
		Code:    500,
		Message: message,
		Details: err.Error(),
	}
}

func FormatError(err error) string {
	if err == nil {
		return ""
	}
	return fmt.Sprintf("%v", err)
}
