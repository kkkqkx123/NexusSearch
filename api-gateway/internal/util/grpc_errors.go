package util

import (
	"net/http"

	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"
)

// GRPCError represents a gRPC error with HTTP status mapping
type GRPCError struct {
	Code       codes.Code
	Message    string
	Details    string
	HTTPStatus int
}

// ConvertGRPCError converts gRPC error to custom error with HTTP status mapping
func ConvertGRPCError(err error) *GRPCError {
	if err == nil {
		return nil
	}

	if st, ok := status.FromError(err); ok {
		httpStatus := mapGRPCCodeToHTTP(st.Code())
		return &GRPCError{
			Code:       st.Code(),
			Message:    st.Message(),
			Details:    st.Message(),
			HTTPStatus: httpStatus,
		}
	}

	return &GRPCError{
		Code:       codes.Unknown,
		Message:    err.Error(),
		Details:    err.Error(),
		HTTPStatus: http.StatusInternalServerError,
	}
}

// mapGRPCCodeToHTTP maps gRPC status codes to HTTP status codes
func mapGRPCCodeToHTTP(code codes.Code) int {
	switch code {
	case codes.OK:
		return http.StatusOK
	case codes.Canceled:
		return http.StatusRequestTimeout
	case codes.Unknown:
		return http.StatusInternalServerError
	case codes.InvalidArgument:
		return http.StatusBadRequest
	case codes.DeadlineExceeded:
		return http.StatusRequestTimeout
	case codes.NotFound:
		return http.StatusNotFound
	case codes.AlreadyExists:
		return http.StatusConflict
	case codes.PermissionDenied:
		return http.StatusForbidden
	case codes.ResourceExhausted:
		return http.StatusTooManyRequests
	case codes.FailedPrecondition:
		return http.StatusPreconditionFailed
	case codes.Aborted:
		return http.StatusConflict
	case codes.OutOfRange:
		return http.StatusRequestedRangeNotSatisfiable
	case codes.Unimplemented:
		return http.StatusNotImplemented
	case codes.Internal:
		return http.StatusInternalServerError
	case codes.Unavailable:
		return http.StatusServiceUnavailable
	case codes.DataLoss:
		return http.StatusInternalServerError
	case codes.Unauthenticated:
		return http.StatusUnauthorized
	default:
		return http.StatusInternalServerError
	}
}

// IsRetryable determines if the error is retryable
func (e *GRPCError) IsRetryable() bool {
	switch e.Code {
	case codes.DeadlineExceeded,
		codes.Unavailable,
		codes.ResourceExhausted,
		codes.Aborted:
		return true
	default:
		return false
	}
}

// Error implements the error interface
func (e *GRPCError) Error() string {
	if e != nil {
		return e.Message
	}
	return ""
}
