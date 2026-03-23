package cache

import (
	"context"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
)

type InvalidationStrategy string

const (
	InvalidationStrategyTime   InvalidationStrategy = "time"
	InvalidationStrategyEvent  InvalidationStrategy = "event"
	InvalidationStrategyManual InvalidationStrategy = "manual"
)

type InvalidationRule struct {
	Pattern   string
	Strategy InvalidationStrategy
	TTL      time.Duration
	Callback func(ctx context.Context, key string) error
}

type CacheInvalidator struct {
	client *redis.Client
	rules  []InvalidationRule
	mu     sync.RWMutex
}

func NewCacheInvalidator(client *redis.Client) *CacheInvalidator {
	return &CacheInvalidator{
		client: client,
		rules:  make([]InvalidationRule, 0),
	}
}

func (ci *CacheInvalidator) AddRule(rule InvalidationRule) {
	ci.mu.Lock()
	defer ci.mu.Unlock()
	ci.rules = append(ci.rules, rule)
}

func (ci *CacheInvalidator) AddRules(rules []InvalidationRule) {
	ci.mu.Lock()
	defer ci.mu.Unlock()
	ci.rules = append(ci.rules, rules...)
}

func (ci *CacheInvalidator) Invalidate(ctx context.Context, key string) error {
	ci.mu.RLock()
	defer ci.mu.RUnlock()

	for _, rule := range ci.rules {
		if matchPattern(key, rule.Pattern) {
			if err := ci.applyRule(ctx, key, rule); err != nil {
				log.Printf("Failed to apply invalidation rule for key %s: %v", key, err)
				continue
			}
		}
	}

	return nil
}

func (ci *CacheInvalidator) InvalidatePattern(ctx context.Context, pattern string) error {
	keys, err := ci.client.Keys(ctx, pattern).Result()
	if err != nil {
		return fmt.Errorf("failed to get keys matching pattern %s: %w", pattern, err)
	}

	for _, key := range keys {
		if err := ci.Invalidate(ctx, key); err != nil {
			log.Printf("Failed to invalidate key %s: %v", key, err)
		}
	}

	return nil
}

func (ci *CacheInvalidator) InvalidateMultiple(ctx context.Context, keys []string) error {
	for _, key := range keys {
		if err := ci.Invalidate(ctx, key); err != nil {
			log.Printf("Failed to invalidate key %s: %v", key, err)
		}
	}

	return nil
}

func (ci *CacheInvalidator) applyRule(ctx context.Context, key string, rule InvalidationRule) error {
	switch rule.Strategy {
	case InvalidationStrategyTime:
		return ci.applyTimeBasedInvalidation(ctx, key, rule)
	case InvalidationStrategyEvent:
		return ci.applyEventBasedInvalidation(ctx, key, rule)
	case InvalidationStrategyManual:
		return ci.applyManualInvalidation(ctx, key, rule)
	default:
		return fmt.Errorf("unknown invalidation strategy: %s", rule.Strategy)
	}
}

func (ci *CacheInvalidator) applyTimeBasedInvalidation(ctx context.Context, key string, rule InvalidationRule) error {
	return ci.client.Expire(ctx, key, rule.TTL).Err()
}

func (ci *CacheInvalidator) applyEventBasedInvalidation(ctx context.Context, key string, rule InvalidationRule) error {
	if rule.Callback != nil {
		return rule.Callback(ctx, key)
	}
	return ci.client.Del(ctx, key).Err()
}

func (ci *CacheInvalidator) applyManualInvalidation(ctx context.Context, key string, rule InvalidationRule) error {
	return ci.client.Del(ctx, key).Err()
}

func matchPattern(key, pattern string) bool {
	if pattern == "*" {
		return true
	}

	if strings.HasSuffix(pattern, "*") {
		prefix := strings.TrimSuffix(pattern, "*")
		return strings.HasPrefix(key, prefix)
	}

	if strings.HasPrefix(pattern, "*") {
		suffix := strings.TrimPrefix(pattern, "*")
		return strings.HasSuffix(key, suffix)
	}

	return key == pattern
}

func (ci *CacheInvalidator) ClearRules() {
	ci.mu.Lock()
	defer ci.mu.Unlock()
	ci.rules = make([]InvalidationRule, 0)
}

func (ci *CacheInvalidator) RuleCount() int {
	ci.mu.RLock()
	defer ci.mu.RUnlock()
	return len(ci.rules)
}
