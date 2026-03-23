package cache

import (
	"context"
	"crypto/md5"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"time"

	"github.com/flexsearch/coordinator/internal/model"
	"github.com/flexsearch/coordinator/internal/util"
	"github.com/redis/go-redis/v9"
)

type Cache interface {
	Get(ctx context.Context, key string) ([]byte, bool)
	Set(ctx context.Context, key string, value []byte, ttl time.Duration) error
	Delete(ctx context.Context, key string) error
	Clear(ctx context.Context) error
	GetStats() *model.CacheStats
}

type RedisCache struct {
	client     *redis.Client
	logger     *util.Logger
	defaultTTL time.Duration
	stats      *model.CacheStats
	enabled    bool
}

type CacheConfig struct {
	Enabled    bool
	Host       string
	Port       int
	Password   string
	DB         int
	PoolSize   int
	DefaultTTL time.Duration
}

func NewRedisCache(config *CacheConfig, logger *util.Logger) (*RedisCache, error) {
	if !config.Enabled {
		return &RedisCache{
			logger:     logger,
			defaultTTL: config.DefaultTTL,
			stats:      &model.CacheStats{},
			enabled:    false,
		}, nil
	}

	client := redis.NewClient(&redis.Options{
		Addr:     fmt.Sprintf("%s:%d", config.Host, config.Port),
		Password: config.Password,
		DB:       config.DB,
		PoolSize: config.PoolSize,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := client.Ping(ctx).Err(); err != nil {
		return nil, fmt.Errorf("failed to connect to Redis: %w", err)
	}

	cache := &RedisCache{
		client:     client,
		logger:     logger,
		defaultTTL: config.DefaultTTL,
		stats:      &model.CacheStats{},
		enabled:    true,
	}

	logger.Info("Redis cache initialized successfully")
	return cache, nil
}

func (c *RedisCache) Get(ctx context.Context, key string) ([]byte, bool) {
	if !c.enabled {
		return nil, false
	}

	val, err := c.client.Get(ctx, key).Bytes()
	if err != nil {
		if err != redis.Nil {
			c.logger.Errorf("Cache get error: %v", err)
		}
		c.stats.Misses++
		return nil, false
	}

	c.stats.Hits++
	c.updateHitRate()
	c.logger.Debugf("Cache hit for key: %s", key)
	return val, true
}

func (c *RedisCache) Set(ctx context.Context, key string, value []byte, ttl time.Duration) error {
	if !c.enabled {
		return nil
	}

	if ttl <= 0 {
		ttl = c.defaultTTL
	}

	if err := c.client.Set(ctx, key, value, ttl).Err(); err != nil {
		c.logger.Errorf("Cache set error: %v", err)
		return err
	}

	c.stats.Size++
	c.logger.Debugf("Cache set for key: %s, TTL: %v", key, ttl)
	return nil
}

func (c *RedisCache) Delete(ctx context.Context, key string) error {
	if !c.enabled {
		return nil
	}

	if err := c.client.Del(ctx, key).Err(); err != nil {
		c.logger.Errorf("Cache delete error: %v", err)
		return err
	}

	c.logger.Debugf("Cache deleted key: %s", key)
	return nil
}

func (c *RedisCache) Clear(ctx context.Context) error {
	if !c.enabled {
		return nil
	}

	if err := c.client.FlushDB(ctx).Err(); err != nil {
		c.logger.Errorf("Cache clear error: %v", err)
		return err
	}

	c.stats.Size = 0
	c.logger.Info("Cache cleared")
	return nil
}

func (c *RedisCache) GetStats() *model.CacheStats {
	c.updateHitRate()
	return c.stats
}

func (c *RedisCache) updateHitRate() {
	total := c.stats.Hits + c.stats.Misses
	if total > 0 {
		c.stats.HitRate = float64(c.stats.Hits) / float64(total)
	}
}

func (c *RedisCache) GenerateCacheKey(req *model.SearchRequest) string {
	keyData := map[string]interface{}{
		"query":   req.Query,
		"index":   req.Index,
		"limit":   req.Limit,
		"offset":  req.Offset,
		"engines": req.Engines,
		"filters": req.Filters,
	}

	jsonData, _ := json.Marshal(keyData)
	hash := md5.Sum(jsonData)
	return fmt.Sprintf("search:%s", hex.EncodeToString(hash[:]))
}

func (c *RedisCache) GetSearchResponse(ctx context.Context, req *model.SearchRequest) (*model.SearchResponse, bool) {
	key := c.GenerateCacheKey(req)
	data, found := c.Get(ctx, key)
	if !found {
		return nil, false
	}

	var response model.SearchResponse
	if err := json.Unmarshal(data, &response); err != nil {
		c.logger.Errorf("Failed to unmarshal cached response: %v", err)
		return nil, false
	}

	response.CacheHit = true
	return &response, true
}

func (c *RedisCache) SetSearchResponse(ctx context.Context, req *model.SearchRequest, response *model.SearchResponse, ttl time.Duration) error {
	key := c.GenerateCacheKey(req)
	data, err := json.Marshal(response)
	if err != nil {
		return fmt.Errorf("failed to marshal response: %w", err)
	}

	return c.Set(ctx, key, data, ttl)
}

func (c *RedisCache) DeleteByPrefix(ctx context.Context, prefix string) error {
	if !c.enabled {
		return nil
	}

	iter := c.client.Scan(ctx, 0, prefix+"*", 0).Iterator()
	var keys []string

	for iter.Next(ctx) {
		keys = append(keys, iter.Val())
	}

	if err := iter.Err(); err != nil {
		return fmt.Errorf("failed to scan keys: %w", err)
	}

	if len(keys) > 0 {
		if err := c.client.Del(ctx, keys...).Err(); err != nil {
			return fmt.Errorf("failed to delete keys: %w", err)
		}
		c.stats.Size -= int64(len(keys))
	}

	c.logger.Debugf("Deleted %d keys with prefix: %s", len(keys), prefix)
	return nil
}

func (c *RedisCache) Warmup(ctx context.Context, queries []string, index string) error {
	if !c.enabled {
		return nil
	}

	c.logger.Infof("Starting cache warmup for %d queries", len(queries))
	
	for i, query := range queries {
		req := &model.SearchRequest{
			Query: query,
			Index: index,
			Limit: 10,
		}
		
		key := c.GenerateCacheKey(req)
		
		if exists, _ := c.client.Exists(ctx, key).Result(); exists > 0 {
			continue
		}
		
		if i%100 == 0 {
			c.logger.Debugf("Cache warmup progress: %d/%d", i, len(queries))
		}
	}

	c.logger.Info("Cache warmup completed")
	return nil
}

func (c *RedisCache) Close() error {
	if c.client != nil {
		return c.client.Close()
	}
	return nil
}

func (c *RedisCache) IsEnabled() bool {
	return c.enabled
}
