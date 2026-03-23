package redis

import (
	"context"
	"time"
)

type HealthChecker struct {
	client *Client
}

func NewHealthChecker(client *Client) *HealthChecker {
	return &HealthChecker{client: client}
}

func (hc *HealthChecker) Check(ctx context.Context) error {
	return hc.client.Ping(ctx).Err()
}

func (hc *HealthChecker) CheckWithTimeout(timeout time.Duration) error {
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	return hc.Check(ctx)
}

func (hc *HealthChecker) GetInfo(ctx context.Context) (string, error) {
	return hc.client.Info(ctx).Result()
}

func (hc *HealthChecker) GetDBSize(ctx context.Context) (int64, error) {
	return hc.client.DBSize(ctx).Result()
}

func (hc *HealthChecker) GetMemoryInfo(ctx context.Context) (string, error) {
	return hc.client.Info(ctx, "memory").Result()
}

func (hc *HealthChecker) GetStats(ctx context.Context) (string, error) {
	return hc.client.Info(ctx, "stats").Result()
}

func (hc *HealthChecker) GetReplicationInfo(ctx context.Context) (string, error) {
	return hc.client.Info(ctx, "replication").Result()
}
