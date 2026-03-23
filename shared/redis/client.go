package redis

import (
	"context"
	"fmt"
	"time"

	"github.com/redis/go-redis/v9"
)

type Client struct {
	*redis.Client
	config *Config
}

func NewClient(config *Config) (*Client, error) {
	if config == nil {
		config = DefaultConfig()
	}

	if err := config.Validate(); err != nil {
		return nil, fmt.Errorf("invalid redis config: %w", err)
	}

	client := redis.NewClient(&redis.Options{
		Addr:            config.Addr(),
		Password:        config.Password,
		DB:              config.DB,
		PoolSize:        config.PoolSize,
		MinIdleConns:    config.MinIdleConns,
		MaxRetries:      config.MaxRetries,
		DialTimeout:     config.DialTimeout,
		ReadTimeout:     config.ReadTimeout,
		WriteTimeout:    config.WriteTimeout,
		PoolTimeout:     config.PoolTimeout,
		ConnMaxIdleTime: config.IdleTimeout,
	})

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	if err := client.Ping(ctx).Err(); err != nil {
		return nil, fmt.Errorf("failed to connect to redis: %w", err)
	}

	return &Client{
		Client: client,
		config: config,
	}, nil
}

func (c *Client) Config() *Config {
	return c.config
}

func (c *Client) Close() error {
	return c.Client.Close()
}

func (c *Client) HealthCheck(ctx context.Context) error {
	return c.Client.Ping(ctx).Err()
}

func (c *Client) GetInfo(ctx context.Context) (string, error) {
	return c.Client.Info(ctx).Result()
}
