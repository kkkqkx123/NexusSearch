package middleware

import (
	"context"
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/flexsearch/api-gateway/internal/util"
	"github.com/gin-gonic/gin"
)

type RateLimitConfig struct {
	Enabled       bool
	DefaultLimit  int
	DefaultBurst  int
	DefaultWindow string
	ByUser        bool
	ByIP          bool
	HeaderBased   bool
	HeaderName    string
	TierHeader    string
}

func RateLimitMiddleware(limiter *util.RateLimiter, config RateLimitConfig) gin.HandlerFunc {
	return func(c *gin.Context) {
		if !config.Enabled {
			c.Next()
			return
		}

		key := determineRateLimitKey(c, config)
		tier := determineUserTier(c, config)

		allowed, err := limiter.Allow(c.Request.Context(), key, tier)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{
				"error":   "Rate limit error",
				"details": err.Error(),
			})
			c.Abort()
			return
		}

		if !allowed {
			tierConfig, exists := limiter.GetConfig().Tiers[tier]
			if !exists {
				tierConfig = util.TierConfig{
					Limit:  limiter.GetConfig().DefaultLimit,
					Burst:  limiter.GetConfig().DefaultBurst,
					Window: limiter.GetConfig().DefaultWindow,
				}
			}

			c.JSON(http.StatusTooManyRequests, gin.H{
				"error":       "Rate limit exceeded",
				"limit":       tierConfig.Limit,
				"burst":       tierConfig.Burst,
				"window":      tierConfig.Window.String(),
				"tier":        string(tier),
				"retry_after": tierConfig.Window.Seconds(),
			})
			c.Abort()
			return
		}

		tierConfig, exists := limiter.GetConfig().Tiers[tier]
		if !exists {
			tierConfig = util.TierConfig{
				Limit:  limiter.GetConfig().DefaultLimit,
				Burst:  limiter.GetConfig().DefaultBurst,
				Window: limiter.GetConfig().DefaultWindow,
			}
		}

		c.Header("X-RateLimit-Limit", fmt.Sprintf("%d", tierConfig.Limit))
		c.Header("X-RateLimit-Remaining", fmt.Sprintf("%d", getRemainingTokens(c.Request.Context(), limiter, key, tier)))
		c.Header("X-RateLimit-Reset", getResetTime(tierConfig.Window))
		c.Header("X-RateLimit-Tier", string(tier))

		c.Next()
	}
}

func determineRateLimitKey(c *gin.Context, config RateLimitConfig) string {
	if config.HeaderBased && config.HeaderName != "" {
		if headerValue := c.GetHeader(config.HeaderName); headerValue != "" {
			return fmt.Sprintf("header:%s:%s", config.HeaderName, headerValue)
		}
	}

	if config.ByUser {
		if userID := c.GetString("user_id"); userID != "" {
			return fmt.Sprintf("user:%s", userID)
		}
	}

	if config.ByIP {
		return fmt.Sprintf("ip:%s", c.ClientIP())
	}

	return "global"
}

func determineUserTier(c *gin.Context, config RateLimitConfig) util.RateLimitTier {
	if config.TierHeader != "" {
		if tierStr := c.GetHeader(config.TierHeader); tierStr != "" {
			tier := util.RateLimitTier(strings.ToLower(tierStr))
			if isValidTier(tier) {
				return tier
			}
		}
	}

	if userTier := c.GetString("rate_limit_tier"); userTier != "" {
		tier := util.RateLimitTier(strings.ToLower(userTier))
		if isValidTier(tier) {
			return tier
		}
	}

	if roles := c.GetStringSlice("user_roles"); len(roles) > 0 {
		for _, role := range roles {
			if strings.Contains(strings.ToLower(role), "enterprise") {
				return util.TierEnterprise
			} else if strings.Contains(strings.ToLower(role), "premium") {
				return util.TierPremium
			} else if strings.Contains(strings.ToLower(role), "basic") {
				return util.TierBasic
			}
		}
	}

	return util.TierFree
}

func isValidTier(tier util.RateLimitTier) bool {
	switch tier {
	case util.TierFree, util.TierBasic, util.TierPremium, util.TierEnterprise:
		return true
	default:
		return false
	}
}

func getRemainingTokens(ctx context.Context, limiter *util.RateLimiter, key string, tier util.RateLimitTier) int {
	tierConfig, exists := limiter.GetConfig().Tiers[tier]
	if !exists {
		tierConfig = util.TierConfig{
			Burst: limiter.GetConfig().DefaultBurst,
		}
	}

	stats, err := limiter.GetStats(ctx, key)
	if err != nil || !stats["exists"].(bool) {
		return tierConfig.Burst
	}

	value, ok := stats["value"].(string)
	if !ok || value == "" {
		return tierConfig.Burst
	}

	parts := []byte(value)
	if len(parts) >= 16 {
		tokens := int(int64(parts[0]) | int64(parts[1])<<8 | int64(parts[2])<<16 | int64(parts[3])<<24)
		if tokens < 0 {
			tokens = 0
		}
		return tokens
	}

	return tierConfig.Burst
}

func getResetTime(window time.Duration) string {
	resetTime := time.Now().Add(window).Unix()
	return fmt.Sprintf("%d", resetTime)
}
