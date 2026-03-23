package util

import (
	"context"
	"time"

	"github.com/flexsearch/shared/metrics"
	"github.com/flexsearch/shared/redis"
	goRedis "github.com/redis/go-redis/v9"
)

type RedisClient struct {
	client  *redis.Client
	metrics *metrics.RedisMetrics
}

func NewRedisClient(config *redis.Config) (*RedisClient, error) {
	client, err := redis.NewClient(config)
	if err != nil {
		return nil, err
	}

	return &RedisClient{
		client:  client,
		metrics: metrics.NewRedisMetrics("api-gateway", "default"),
	}, nil
}

func (r *RedisClient) Close() error {
	return r.client.Close()
}

func (r *RedisClient) Ping(ctx context.Context) error {
	return r.client.Ping(ctx).Err()
}

func (r *RedisClient) Get(ctx context.Context, key string) (string, error) {
	start := time.Now()
	result, err := r.client.Get(ctx, key).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil && err != goRedis.Nil {
		status = "error"
	}

	r.metrics.RecordOperation("get", status, duration)
	return result, err
}

func (r *RedisClient) Set(ctx context.Context, key string, value interface{}, expiration time.Duration) error {
	start := time.Now()
	err := r.client.Set(ctx, key, value, expiration).Err()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("set", status, duration)
	return err
}

func (r *RedisClient) Del(ctx context.Context, keys ...string) error {
	start := time.Now()
	err := r.client.Del(ctx, keys...).Err()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("del", status, duration)
	return err
}

func (r *RedisClient) Incr(ctx context.Context, key string) (int64, error) {
	start := time.Now()
	result, err := r.client.Incr(ctx, key).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("incr", status, duration)
	return result, err
}

func (r *RedisClient) Expire(ctx context.Context, key string, expiration time.Duration) error {
	start := time.Now()
	err := r.client.Expire(ctx, key, expiration).Err()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("expire", status, duration)
	return err
}

func (r *RedisClient) TTL(ctx context.Context, key string) (time.Duration, error) {
	start := time.Now()
	result, err := r.client.TTL(ctx, key).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("ttl", status, duration)
	return result, err
}

func (r *RedisClient) ZAdd(ctx context.Context, key string, members ...goRedis.Z) error {
	start := time.Now()
	err := r.client.ZAdd(ctx, key, members...).Err()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("zadd", status, duration)
	return err
}

func (r *RedisClient) ZRemRangeByScore(ctx context.Context, key, min, max string) error {
	start := time.Now()
	err := r.client.ZRemRangeByScore(ctx, key, min, max).Err()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("zremrangebyscore", status, duration)
	return err
}

func (r *RedisClient) ZCount(ctx context.Context, key, min, max string) (int64, error) {
	start := time.Now()
	result, err := r.client.ZCount(ctx, key, min, max).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("zcount", status, duration)
	return result, err
}

func (r *RedisClient) ZRevRangeByScoreWithScores(ctx context.Context, key string, opt *goRedis.ZRangeBy) ([]goRedis.Z, error) {
	start := time.Now()
	result, err := r.client.ZRevRangeByScoreWithScores(ctx, key, opt).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("zrevrangebyscorewithscores", status, duration)
	return result, err
}

func (r *RedisClient) Eval(ctx context.Context, script string, keys []string, args ...interface{}) (interface{}, error) {
	start := time.Now()
	result, err := r.client.Eval(ctx, script, keys, args...).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("eval", status, duration)
	return result, err
}

func (r *RedisClient) ScriptLoad(ctx context.Context, script string) (string, error) {
	start := time.Now()
	result, err := r.client.ScriptLoad(ctx, script).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("scriptload", status, duration)
	return result, err
}

func (r *RedisClient) EvalSha(ctx context.Context, sha1 string, keys []string, args ...interface{}) (interface{}, error) {
	start := time.Now()
	result, err := r.client.EvalSha(ctx, sha1, keys, args...).Result()
	duration := time.Since(start).Seconds()

	status := "success"
	if err != nil {
		status = "error"
	}

	r.metrics.RecordOperation("evalsha", status, duration)
	return result, err
}

func (r *RedisClient) Client() *redis.Client {
	return r.client
}
