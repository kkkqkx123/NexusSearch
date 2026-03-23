# 第2阶段技术选型分析

## 概述

本文档分析了第2阶段（核心功能实现）所需的技术库选型，包括 JWT 认证、限流、熔断器等关键功能的实现方案。

---

## 1. JWT 认证

### 选型库：golang-jwt/jwt

**库信息**：
- 库名：golang-jwt/jwt
- 版本：v5
- 源声誉：Medium
- 代码片段数：61
- Benchmark Score：73.2

**选型理由**：
1. Go 官方推荐的 JWT 库
2. 支持多种签名算法（HMAC、RSA、ECDSA）
3. 活跃的维护和社区支持
4. 完善的文档和示例

### 实现方案

#### 1.1 自定义 Claims 结构

```go
type CustomClaims struct {
    UserID   string   `json:"user_id"`
    Username string   `json:"username"`
    Role     string   `json:"role"`
    jwt.RegisteredClaims
}
```

#### 1.2 Token 生成

```go
func GenerateToken(userID, username, role string) (string, error) {
    mySigningKey := []byte(config.JWT.Secret)

    claims := CustomClaims{
        UserID:   userID,
        Username: username,
        Role:     role,
        RegisteredClaims: jwt.RegisteredClaims{
            Issuer:    "api-gateway",
            Subject:   userID,
            ExpiresAt: jwt.NewNumericDate(time.Now().Add(time.Hour * 24)),
            IssuedAt:  jwt.NewNumericDate(time.Now()),
        },
    }

    token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
    return token.SignedString(mySigningKey)
}
```

#### 1.3 Token 验证中间件

```go
func AuthMiddleware() gin.HandlerFunc {
    return func(c *gin.Context) {
        authHeader := c.GetHeader("Authorization")
        if authHeader == "" {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "Missing authorization header"})
            c.Abort()
            return
        }

        tokenString := strings.TrimPrefix(authHeader, "Bearer ")
        if tokenString == authHeader {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid authorization format"})
            c.Abort()
            return
        }

        token, err := jwt.ParseWithClaims(tokenString, &CustomClaims{}, func(token *jwt.Token) (interface{}, error) {
            if _, ok := token.Method.(*jwt.SigningMethodHMAC); !ok {
                return nil, fmt.Errorf("unexpected signing method: %v", token.Header["alg"])
            }
            return []byte(config.JWT.Secret), nil
        })

        if err != nil || !token.Valid {
            c.JSON(http.StatusUnauthorized, gin.H{"error": "Invalid token"})
            c.Abort()
            return
        }

        if claims, ok := token.Claims.(*CustomClaims); ok {
            c.Set("user_id", claims.UserID)
            c.Set("username", claims.Username)
            c.Set("role", claims.Role)
        }

        c.Next()
    }
}
```

### 配置项

```yaml
jwt:
  secret: your-256-bit-secret
  expiration: 24h
  issuer: api-gateway
```

---

## 2. 限流

### 选型库：go-redis/redis

**库信息**：
- 库名：redis/go-redis
- 版本：v9
- 源声誉：High
- 代码片段数：58
- Benchmark Score：92.7

**选型理由**：
1. Go 官方 Redis 客户端
2. 高性能和稳定性
3. 支持连接池、管道、事务
4. 支持分布式限流

### 实现方案

#### 2.1 滑动窗口算法

使用 Redis 的有序集合（ZSET）实现滑动窗口限流：

```go
type RateLimiter struct {
    redis *redis.Client
}

func NewRateLimiter(redis *redis.Client) *RateLimiter {
    return &RateLimiter{redis: redis}
}

func (rl *RateLimiter) Allow(ctx context.Context, key string, limit int, window time.Duration) (bool, error) {
    now := time.Now().Unix()
    windowStart := now - int64(window.Seconds())

    pipe := rl.redis.Pipeline()

    pipe.ZRemRangeByScore(ctx, key, "0", strconv.FormatInt(windowStart, 10))
    pipe.ZCard(ctx, key)
    pipe.ZAdd(ctx, key, redis.Z{Score: float64(now), Member: now})
    pipe.Expire(ctx, key, window)

    results, err := pipe.Exec(ctx)
    if err != nil {
        return false, err
    }

    count := results[1].(*redis.IntCmd).Val()
    if count >= int64(limit) {
        return false, nil
    }

    return true, nil
}
```

#### 2.2 限流中间件

```go
func RateLimitMiddleware(limiter *RateLimiter, limit int, window time.Duration) gin.HandlerFunc {
    return func(c *gin.Context) {
        userID := c.GetString("user_id")
        if userID == "" {
            userID = c.ClientIP()
        }

        key := fmt.Sprintf("ratelimit:%s", userID)
        allowed, err := limiter.Allow(c.Request.Context(), key, limit, window)
        if err != nil {
            c.JSON(http.StatusInternalServerError, gin.H{"error": "Rate limit error"})
            c.Abort()
            return
        }

        if !allowed {
            c.JSON(http.StatusTooManyRequests, gin.H{
                "error": "Rate limit exceeded",
                "limit": limit,
                "window": window.String(),
            })
            c.Abort()
            return
        }

        c.Next()
    }
}
```

### 配置项

```yaml
ratelimit:
  enabled: true
  default_limit: 100
  default_window: 1m
  by_user: true
  by_ip: true
```

---

## 3. 熔断器

### 选型库：afex/hystrix-go

**库信息**：
- 库名：afex/hystrix-go
- 源声誉：High
- 代码片段数：13
- Benchmark Score：N/A

**选型理由**：
1. Netflix Hystrix 的 Go 实现
2. 成熟的熔断器模式
3. 支持降级和超时
4. 简单易用的 API

### 实现方案

#### 3.1 熔断器配置

```go
func ConfigureCircuitBreaker(name string, timeout int, maxConcurrent int, errorPercent int) {
    hystrix.ConfigureCommand(name, hystrix.CommandConfig{
        Timeout:                timeout,
        MaxConcurrentRequests:   maxConcurrent,
        ErrorPercentThreshold:   errorPercent,
        RequestVolumeThreshold:  10,
        SleepWindow:            5000,
    })
}
```

#### 3.2 熔断器包装器

```go
type CircuitBreaker struct {
    name string
}

func NewCircuitBreaker(name string) *CircuitBreaker {
    return &CircuitBreaker{name: name}
}

func (cb *CircuitBreaker) Do(run func() error, fallback func(error) error) error {
    return hystrix.Do(cb.name, run, fallback)
}
```

#### 3.3 在处理器中使用

```go
func (h *SearchHandler) Search(c *gin.Context) {
    cb := NewCircuitBreaker("search_service")

    err := cb.Do(
        func() error {
            return h.callCoordinator(c)
        },
        func(err error) error {
            return h.fallbackSearch(c)
        },
    )

    if err != nil {
        c.JSON(http.StatusServiceUnavailable, gin.H{"error": "Service unavailable"})
        return
    }
}
```

### 配置项

```yaml
circuitbreaker:
  enabled: true
  default_timeout: 1000
  default_max_concurrent: 100
  default_error_percent: 25
  commands:
    search_service:
      timeout: 1000
      max_concurrent: 100
      error_percent: 25
    document_service:
      timeout: 2000
      max_concurrent: 50
      error_percent: 30
```

---

## 4. CORS 中间件

### 实现方案

使用 Gin 的 CORS 中间件：

```go
func CORSMiddleware() gin.HandlerFunc {
    return func(c *gin.Context) {
        c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
        c.Writer.Header().Set("Access-Control-Allow-Credentials", "true")
        c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Content-Length, Accept-Encoding, X-CSRF-Token, Authorization, accept, origin, Cache-Control, X-Requested-With")
        c.Writer.Header().Set("Access-Control-Allow-Methods", "POST, OPTIONS, GET, PUT, DELETE")

        if c.Request.Method == "OPTIONS" {
            c.AbortWithStatus(http.StatusNoContent)
            return
        }

        c.Next()
    }
}
```

### 配置项

```yaml
cors:
  enabled: true
  allow_origins: "*"
  allow_methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
  allow_headers: ["Content-Type", "Authorization"]
  allow_credentials: true
```

---

## 5. 参数验证

### 选型库：go-playground/validator

**库信息**：
- 库名：go-playground/validator
- 版本：v10
- 源声誉：High

**选型理由**：
1. Gin 默认集成的验证器
2. 支持结构体标签验证
3. 支持自定义验证规则
4. 丰富的验证规则

### 实现方案

#### 5.1 请求模型

```go
type SearchRequest struct {
    Query   string            `json:"query" binding:"required,min=1,max=100"`
    Limit   int               `json:"limit" binding:"min=1,max=100"`
    Offset  int               `json:"offset" binding:"min=0"`
    Filters map[string]string  `json:"filters"`
    Options map[string]interface{} `json:"options"`
}

type CreateDocumentRequest struct {
    ID      string                 `json:"id" binding:"required"`
    Title   string                 `json:"title" binding:"required,min=1,max=200"`
    Content string                 `json:"content" binding:"required,min=1"`
    Fields  map[string]interface{}  `json:"fields"`
}
```

#### 5.2 验证中间件

```go
func ValidateMiddleware() gin.HandlerFunc {
    return func(c *gin.Context) {
        if err := c.ShouldBindJSON(c.Request.Body); err != nil {
            c.JSON(http.StatusBadRequest, gin.H{
                "error": "Validation failed",
                "details": err.Error(),
            })
            c.Abort()
            return
        }
        c.Next()
    }
}
```

---

## 6. 错误处理

### 实现方案

#### 6.1 错误类型

```go
type AppError struct {
    Code    int    `json:"code"`
    Message string `json:"message"`
    Details string `json:"details,omitempty"`
}

func (e *AppError) Error() string {
    return e.Message
}

var (
    ErrUnauthorized      = &AppError{Code: 401, Message: "Unauthorized"}
    ErrForbidden        = &AppError{Code: 403, Message: "Forbidden"}
    ErrNotFound        = &AppError{Code: 404, Message: "Not found"}
    ErrRateLimitExceeded = &AppError{Code: 429, Message: "Rate limit exceeded"}
    ErrInternalServer   = &AppError{Code: 500, Message: "Internal server error"}
)
```

#### 6.2 错误处理中间件

```go
func ErrorHandlerMiddleware() gin.HandlerFunc {
    return func(c *gin.Context) {
        c.Next()

        if len(c.Errors) > 0 {
            err := c.Errors.Last().Err

            if appErr, ok := err.(*AppError); ok {
                c.JSON(appErr.Code, gin.H{
                    "error": appErr.Message,
                    "details": appErr.Details,
                })
            } else {
                c.JSON(http.StatusInternalServerError, gin.H{
                    "error": "Internal server error",
                })
            }
        }
    }
}
```

---

## 7. 依赖清单

### Go 依赖更新

```go
require (
    github.com/gin-gonic/gin v1.11.0
    github.com/golang-jwt/jwt/v5 v5.0.0
    github.com/redis/go-redis/v9 v9.0.5
    github.com/afex/hystrix-go v0.0.0-20160520155838-8d45e150d1b3
    github.com/go-playground/validator/v10 v10.15.3
    github.com/spf13/viper v1.21.0
    github.com/google/uuid v1.6.0
)
```

---

## 8. 实施步骤

### 8.1 安装依赖

```bash
go get github.com/golang-jwt/jwt/v5
go get github.com/redis/go-redis/v9
go get github.com/afex/hystrix-go
go get github.com/go-playground/validator/v10
go mod tidy
```

### 8.2 创建文件结构

```
services/api-gateway/
├── internal/
│   ├── middleware/
│   │   ├── auth.go         # JWT 认证中间件
│   │   ├── ratelimit.go    # 限流中间件
│   │   ├── cors.go         # CORS 中间件
│   │   ├── recovery.go     # 恢复中间件
│   │   └── error.go       # 错误处理中间件
│   ├── util/
│   │   ├── jwt.go         # JWT 工具
│   │   ├── ratelimit.go   # 限流工具
│   │   ├── error.go       # 错误定义
│   │   └── validator.go   # 验证器
│   └── model/
│       ├── request.go      # 请求模型
│       └── response.go     # 响应模型
```

### 8.3 实现顺序

1. 错误处理和工具函数
2. JWT 认证中间件
3. 限流中间件
4. CORS 和恢复中间件
5. 数据模型和验证器
6. 集成到路由

---

## 9. 测试计划

### 9.1 单元测试

- JWT 生成和验证
- 限流算法正确性
- 熔断器配置和触发
- 参数验证规则

### 9.2 集成测试

- 认证流程端到端测试
- 限流功能测试
- 熔断器降级测试
- 错误处理测试

---

## 10. 注意事项

1. **安全性**：
   - JWT Secret 必须从安全配置中读取
   - 限流 key 需要考虑用户隐私
   - CORS 配置需要根据生产环境调整

2. **性能**：
   - Redis 连接池大小需要合理配置
   - 熔断器超时时间需要根据实际业务调整
   - 限流窗口大小需要平衡用户体验和系统保护

3. **可观测性**：
   - 记录限流事件
   - 记录熔断器状态变化
   - 记录认证失败事件

---

## 总结

本技术选型分析为第2阶段提供了完整的技术方案：

1. **JWT 认证**：使用 golang-jwt/jwt v5
2. **限流**：使用 go-redis/redis v9 实现滑动窗口
3. **熔断器**：使用 afex/hystrix-go
4. **参数验证**：使用 go-playground/validator v10
5. **错误处理**：自定义错误类型和中间件

所有选型的库都有良好的社区支持和文档，可以快速实现第2阶段的核心功能。
