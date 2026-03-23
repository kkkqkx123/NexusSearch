package util

import (
	"testing"
	"time"
)

func TestJWTManager_GenerateToken(t *testing.T) {
	jwtManager := NewJWTManager("test-secret-key", "test-issuer", 24)

	token, err := jwtManager.GenerateToken("user123", "testuser", "admin")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	if token == "" {
		t.Fatal("Generated token is empty")
	}

	if len(token) < 50 {
		t.Fatalf("Token seems too short: %s", token)
	}
}

func TestJWTManager_ValidateToken(t *testing.T) {
	jwtManager := NewJWTManager("test-secret-key", "test-issuer", 24)

	token, err := jwtManager.GenerateToken("user123", "testuser", "admin")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	claims, err := jwtManager.ValidateToken(token)
	if err != nil {
		t.Fatalf("Failed to validate token: %v", err)
	}

	if claims.UserID != "user123" {
		t.Errorf("Expected UserID 'user123', got '%s'", claims.UserID)
	}

	if claims.Username != "testuser" {
		t.Errorf("Expected Username 'testuser', got '%s'", claims.Username)
	}

	if claims.Role != "admin" {
		t.Errorf("Expected Role 'admin', got '%s'", claims.Role)
	}
}

func TestJWTManager_ValidateToken_Invalid(t *testing.T) {
	jwtManager := NewJWTManager("test-secret-key", "test-issuer", 24)

	_, err := jwtManager.ValidateToken("invalid-token")
	if err == nil {
		t.Fatal("Expected error for invalid token, got nil")
	}
}

func TestJWTManager_ValidateToken_WrongSecret(t *testing.T) {
	jwtManager1 := NewJWTManager("secret-key-1", "test-issuer", 24)
	jwtManager2 := NewJWTManager("secret-key-2", "test-issuer", 24)

	token, err := jwtManager1.GenerateToken("user123", "testuser", "admin")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	_, err = jwtManager2.ValidateToken(token)
	if err == nil {
		t.Fatal("Expected error for token signed with different secret")
	}
}

func TestJWTManager_TokenExpiration(t *testing.T) {
	jwtManager := NewJWTManager("test-secret-key", "test-issuer", 24)

	token, err := jwtManager.GenerateToken("user123", "testuser", "admin")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	claims, err := jwtManager.ValidateToken(token)
	if err != nil {
		t.Fatalf("Failed to validate token: %v", err)
	}

	if claims.ExpiresAt == nil {
		t.Fatal("ExpiresAt claim is nil")
	}

	expectedExpiry := time.Now().Add(24 * time.Hour)
	actualExpiry := claims.ExpiresAt.Time

	if actualExpiry.Before(expectedExpiry.Add(-time.Minute)) || actualExpiry.After(expectedExpiry.Add(time.Minute)) {
		t.Errorf("Token expiration time is not as expected: expected around %v, got %v", expectedExpiry, actualExpiry)
	}
}

func TestJWTManager_RefreshToken(t *testing.T) {
	jwtManager := NewJWTManager("test-secret-key", "test-issuer", 24)

	token1, err := jwtManager.GenerateToken("user123", "testuser", "admin")
	if err != nil {
		t.Fatalf("Failed to generate token: %v", err)
	}

	token2, err := jwtManager.RefreshToken(token1)
	if err != nil {
		t.Fatalf("Failed to refresh token: %v", err)
	}

	if token2 == "" {
		t.Fatal("Refreshed token should not be empty")
	}

	claims, err := jwtManager.ValidateToken(token2)
	if err != nil {
		t.Fatalf("Failed to validate refreshed token: %v", err)
	}

	if claims.UserID != "user123" {
		t.Errorf("Expected UserID 'user123', got '%s'", claims.UserID)
	}

	if claims.Username != "testuser" {
		t.Errorf("Expected Username 'testuser', got '%s'", claims.Username)
	}

	if claims.Role != "admin" {
		t.Errorf("Expected Role 'admin', got '%s'", claims.Role)
	}
}

func TestJWTManager_InvalidTokenFormat(t *testing.T) {
	jwtManager := NewJWTManager("test-secret-key", "test-issuer", 24)

	testCases := []string{
		"",
		"invalid",
		"a.b",
		"a.b.c.d",
	}

	for _, tc := range testCases {
		_, err := jwtManager.ValidateToken(tc)
		if err == nil {
			t.Errorf("Expected error for token '%s', got nil", tc)
		}
	}
}
