package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/gin-gonic/gin"
)

func TestCORSMiddleware_OptionsRequest(t *testing.T) {
	config := CORSConfig{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE"},
		AllowHeaders:     []string{"Content-Type", "Authorization"},
		AllowCredentials: true,
	}

	router := gin.New()
	router.Use(CORSMiddleware(config))
	router.OPTIONS("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodOptions, "/test", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusNoContent {
		t.Errorf("Expected status %d for OPTIONS request, got %d", http.StatusNoContent, w.Code)
	}

	if w.Header().Get("Access-Control-Allow-Origin") == "" {
		t.Error("Access-Control-Allow-Origin header should be set")
	}

	if w.Header().Get("Access-Control-Allow-Methods") == "" {
		t.Error("Access-Control-Allow-Methods header should be set")
	}

	if w.Header().Get("Access-Control-Allow-Headers") == "" {
		t.Error("Access-Control-Allow-Headers header should be set")
	}
}

func TestCORSMiddleware_SimpleRequest(t *testing.T) {
	config := CORSConfig{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST"},
		AllowHeaders:     []string{"Content-Type"},
		AllowCredentials: false,
	}

	var handlerCalled bool

	router := gin.New()
	router.Use(CORSMiddleware(config))
	router.GET("/test", func(c *gin.Context) {
		handlerCalled = true
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if !handlerCalled {
		t.Error("Handler was not called")
	}

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	if w.Header().Get("Access-Control-Allow-Origin") != "*" {
		t.Errorf("Expected Access-Control-Allow-Origin '*', got '%s'", w.Header().Get("Access-Control-Allow-Origin"))
	}
}

func TestCORSMiddleware_SpecificOrigins(t *testing.T) {
	config := CORSConfig{
		AllowOrigins:     []string{"http://example.com", "https://api.example.com"},
		AllowMethods:     []string{"GET", "POST"},
		AllowHeaders:     []string{"Content-Type"},
		AllowCredentials: true,
	}

	router := gin.New()
	router.Use(CORSMiddleware(config))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Origin", "https://api.example.com")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Header().Get("Access-Control-Allow-Origin") != "https://api.example.com" {
		t.Errorf("Expected Access-Control-Allow-Origin 'https://api.example.com', got '%s'", w.Header().Get("Access-Control-Allow-Origin"))
	}

	if w.Header().Get("Access-Control-Allow-Credentials") != "true" {
		t.Error("Access-Control-Allow-Credentials should be 'true'")
	}
}

func TestCORSMiddleware_DefaultValues(t *testing.T) {
	config := CORSConfig{
		AllowOrigins:     nil,
		AllowMethods:     nil,
		AllowHeaders:     nil,
		AllowCredentials: false,
	}

	router := gin.New()
	router.Use(CORSMiddleware(config))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Header().Get("Access-Control-Allow-Origin") != "*" {
		t.Errorf("Expected default Access-Control-Allow-Origin '*', got '%s'", w.Header().Get("Access-Control-Allow-Origin"))
	}

	if w.Header().Get("Access-Control-Allow-Methods") == "" {
		t.Error("Default Access-Control-Allow-Methods should be set")
	}

	if w.Header().Get("Access-Control-Allow-Headers") == "" {
		t.Error("Default Access-Control-Allow-Headers should be set")
	}
}
