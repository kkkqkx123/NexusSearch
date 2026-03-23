package util

import (
	"context"
	"fmt"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
)

type RateLimitTier string

const (
	TierFree       RateLimitTier = "free"
	TierBasic      RateLimitTier = "basic"
	TierPremium    RateLimitTier = "premium"
	TierEnterprise RateLimitTier = "enterprise"
)

type RateLimitConfig struct {
	Enabled       bool
	DefaultLimit  int
	DefaultBurst  int
	DefaultWindow time.Duration
	ByUser        bool
	ByIP          bool
	Tiers         map[RateLimitTier]TierConfig
	RedisPrefix   string
}

type TierConfig struct {
	Limit  int
	Burst  int
	Window time.Duration
}

func DefaultRateLimitConfig() RateLimitConfig {
	return RateLimitConfig{
		Enabled:       true,
		DefaultLimit:  100,
		DefaultBurst:  20,
		DefaultWindow: time.Minute,
		ByUser:        true,
		ByIP:          true,
		RedisPrefix:   "ratelimit",
		Tiers: map[RateLimitTier]TierConfig{
			TierFree: {
				Limit:  60,
				Burst:  10,
				Window: time.Minute,
			},
			TierBasic: {
				Limit:  300,
				Burst:  50,
				Window: time.Minute,
			},
			TierPremium: {
				Limit:  1000,
				Burst:  200,
				Window: time.Minute,
			},
			TierEnterprise: {
				Limit:  5000,
				Burst:  1000,
				Window: time.Minute,
			},
		},
	}
}

type RateLimiter struct {
	redis  *redis.Client
	config RateLimitConfig
	mu     sync.RWMutex
}

func NewRateLimiter(redisClient *redis.Client, config RateLimitConfig) *RateLimiter {
	return &RateLimiter{
		redis:  redisClient,
		config: config,
	}
}

func (rl *RateLimiter) Allow(ctx context.Context, key string, tier RateLimitTier) (bool, error) {
	if !rl.config.Enabled {
		return true, nil
	}

	tierConfig, exists := rl.config.Tiers[tier]
	if !exists {
		tierConfig = TierConfig{
			Limit:  rl.config.DefaultLimit,
			Burst:  rl.config.DefaultBurst,
			Window: rl.config.DefaultWindow,
		}
	}

	return rl.allowRequest(ctx, key, tierConfig)
}

func (rl *RateLimiter) allowRequest(ctx context.Context, key string, config TierConfig) (bool, error) {
	bucketKey := fmt.Sprintf("%s:bucket:%s", rl.config.RedisPrefix, key)

	pipe := rl.redis.Pipeline()
	getCmd := pipe.Get(ctx, bucketKey)

	_, err := pipe.Exec(ctx)
	if err != nil && err != redis.Nil {
		return false, err
	}

	var tokens int
	var lastRefill time.Time

	if err == redis.Nil {
		tokens = config.Burst
		lastRefill = time.Now()
	} else {
		value, _ := getCmd.Result()
		if value != "" {
			parts := []byte(value)
			if len(parts) >= 16 {
				tokens = int(int64(parts[0]) | int64(parts[1])<<8 | int64(parts[2])<<16 | int64(parts[3])<<24)
				lastRefill = time.Unix(int64(parts[4])|int64(parts[5])<<8|int64(parts[6])<<16|int64(parts[7])<<24,
					int64(parts[8])|int64(parts[9])<<8|int64(parts[10])<<16|int64(parts[11])<<24)
			}
		}
	}

	now := time.Now()
	elapsed := now.Sub(lastRefill)
	tokensToAdd := int(elapsed.Seconds() * float64(config.Limit) / config.Window.Seconds())

	if tokensToAdd > 0 {
		tokens = min(tokens+tokensToAdd, config.Burst)
		lastRefill = now
	}

	if tokens > 0 {
		tokens--

		value := fmt.Sprintf("%d:%d", tokens, lastRefill.Unix())
		err = rl.redis.Set(ctx, bucketKey, value, config.Window).Err()
		if err != nil {
			return false, err
		}

		return true, nil
	}

	value := fmt.Sprintf("%d:%d", tokens, lastRefill.Unix())
	err = rl.redis.Set(ctx, bucketKey, value, config.Window).Err()
	if err != nil {
		return false, err
	}

	return false, nil
}

func (rl *RateLimiter) GetStats(ctx context.Context, key string) (map[string]interface{}, error) {
	bucketKey := fmt.Sprintf("%s:bucket:%s", rl.config.RedisPrefix, key)

	value, err := rl.redis.Get(ctx, bucketKey).Result()
	if err != nil && err != redis.Nil {
		return nil, err
	}

	stats := map[string]interface{}{
		"key":    key,
		"exists": err != redis.Nil,
	}

	if err == nil && value != "" {
		stats["value"] = value
	}

	return stats, nil
}

func (rl *RateLimiter) GetConfig() RateLimitConfig {
	rl.mu.RLock()
	defer rl.mu.RUnlock()
	return rl.config
}

func (rl *RateLimiter) Reset(ctx context.Context, key string) error {
	bucketKey := fmt.Sprintf("%s:bucket:%s", rl.config.RedisPrefix, key)
	return rl.redis.Del(ctx, bucketKey).Err()
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func GetUserTierFromContext(ctx context.Context, defaultTier RateLimitTier) RateLimitTier {
	return defaultTier
}
