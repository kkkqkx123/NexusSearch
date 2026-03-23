package middleware

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/flexsearch/api-gateway/internal/util"
	"github.com/gin-gonic/gin"
)

func init() {
	gin.SetMode(gin.TestMode)
}

func TestAuthMiddleware_MissingHeader(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	router := gin.New()
	router.Use(AuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusUnauthorized {
		t.Errorf("Expected status %d, got %d", http.StatusUnauthorized, w.Code)
	}
}

func TestAuthMiddleware_InvalidFormat(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	router := gin.New()
	router.Use(AuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Authorization", "InvalidFormat token123")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusUnauthorized {
		t.Errorf("Expected status %d, got %d", http.StatusUnauthorized, w.Code)
	}
}

func TestAuthMiddleware_EmptyToken(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	router := gin.New()
	router.Use(AuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Authorization", "Bearer ")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusUnauthorized {
		t.Errorf("Expected status %d, got %d", http.StatusUnauthorized, w.Code)
	}
}

func TestAuthMiddleware_InvalidToken(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	router := gin.New()
	router.Use(AuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Authorization", "Bearer invalid-token")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusUnauthorized {
		t.Errorf("Expected status %d, got %d", http.StatusUnauthorized, w.Code)
	}
}

func TestAuthMiddleware_ValidToken(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	token, err := jwtManager.GenerateToken("user123", "testuser", "admin")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	var capturedUserID, capturedUsername, capturedRole string

	router := gin.New()
	router.Use(AuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		capturedUserID = c.GetString("user_id")
		capturedUsername = c.GetString("username")
		capturedRole = c.GetString("role")
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	if capturedUserID != "user123" {
		t.Errorf("Expected user_id 'user123', got '%s'", capturedUserID)
	}

	if capturedUsername != "testuser" {
		t.Errorf("Expected username 'testuser', got '%s'", capturedUsername)
	}

	if capturedRole != "admin" {
		t.Errorf("Expected role 'admin', got '%s'", capturedRole)
	}
}

func TestOptionalAuthMiddleware_NoHeader(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	called := false

	router := gin.New()
	router.Use(OptionalAuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		called = true
		userID := c.GetString("user_id")
		if userID != "" {
			t.Error("Expected empty user_id when no auth header")
		}
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	if !called {
		t.Error("Handler was not called")
	}
}

func TestOptionalAuthMiddleware_InvalidToken(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	called := false

	router := gin.New()
	router.Use(OptionalAuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		called = true
		userID := c.GetString("user_id")
		if userID != "" {
			t.Error("Expected empty user_id when invalid token")
		}
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Authorization", "Bearer invalid-token")
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	if !called {
		t.Error("Handler was not called")
	}
}

func TestOptionalAuthMiddleware_ValidToken(t *testing.T) {
	jwtManager := util.NewJWTManager("test-secret", "test-issuer", 24)

	token, err := jwtManager.GenerateToken("user456", "optionaluser", "user")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	var capturedUserID string
	called := false

	router := gin.New()
	router.Use(OptionalAuthMiddleware(jwtManager))
	router.GET("/test", func(c *gin.Context) {
		called = true
		capturedUserID = c.GetString("user_id")
		c.JSON(http.StatusOK, gin.H{"status": "ok"})
	})

	req := httptest.NewRequest(http.MethodGet, "/test", nil)
	req.Header.Set("Authorization", "Bearer "+token)
	w := httptest.NewRecorder()

	router.ServeHTTP(w, req)

	if w.Code != http.StatusOK {
		t.Errorf("Expected status %d, got %d", http.StatusOK, w.Code)
	}

	if !called {
		t.Error("Handler was not called")
	}

	if capturedUserID != "user456" {
		t.Errorf("Expected user_id 'user456', got '%s'", capturedUserID)
	}
}
