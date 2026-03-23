package handler

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
)

func init() {
	gin.SetMode(gin.TestMode)
}

func TestHealthHandler_Check(t *testing.T) {
	handler := NewHealthHandler(nil, nil, nil)

	router := gin.New()
	router.GET("/health", handler.Check)

	req := httptest.NewRequest(http.MethodGet, "/health", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	body := w.Body.String()
	if body == "" {
		t.Error("Response body is empty")
	}

	if !contains(body, "status") {
		t.Error("Response body should contain 'status' field")
	}

	if !contains(body, "healthy") {
		t.Error("Response body should contain 'healthy' status")
	}

	if !contains(body, "service") {
		t.Error("Response body should contain 'service' field")
	}

	if !contains(body, "api-gateway") {
		t.Error("Response body should contain 'api-gateway' service name")
	}
}

func TestHealthHandler_CheckServices(t *testing.T) {
	handler := NewHealthHandler(nil, nil, nil)

	router := gin.New()
	router.GET("/health/services", handler.CheckServices)

	req := httptest.NewRequest(http.MethodGet, "/health/services", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	body := w.Body.String()
	if body == "" {
		t.Error("Response body is empty")
	}

	if !contains(body, "status") {
		t.Error("Response body should contain 'status' field")
	}

	if !contains(body, "healthy") {
		t.Error("Response body should contain 'healthy' status")
	}

	if !contains(body, "services") {
		t.Error("Response body should contain 'services' field")
	}

	if !contains(body, "coordinator") {
		t.Error("Response body should contain 'coordinator' service info")
	}
}

func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(s) > 0 && containsHelper(s, substr))
}

func containsHelper(s, substr string) bool {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}
