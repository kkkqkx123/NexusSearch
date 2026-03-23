package redis

import (
	"fmt"
	"time"
)

type Config struct {
	Host         string        `mapstructure:"host"`
	Port         int           `mapstructure:"port"`
	Password     string        `mapstructure:"password"`
	DB           int           `mapstructure:"db"`
	PoolSize     int           `mapstructure:"pool_size"`
	MinIdleConns int           `mapstructure:"min_idle_conns"`
	MaxRetries   int           `mapstructure:"max_retries"`
	DialTimeout  time.Duration `mapstructure:"dial_timeout"`
	ReadTimeout  time.Duration `mapstructure:"read_timeout"`
	WriteTimeout time.Duration `mapstructure:"write_timeout"`
	PoolTimeout  time.Duration `mapstructure:"pool_timeout"`
	IdleTimeout  time.Duration `mapstructure:"idle_timeout"`
}

func DefaultConfig() *Config {
	return &Config{
		Host:         "localhost",
		Port:         6379,
		Password:     "",
		DB:           0,
		PoolSize:     10,
		MinIdleConns: 2,
		MaxRetries:   3,
		DialTimeout:  5 * time.Second,
		ReadTimeout:  3 * time.Second,
		WriteTimeout: 3 * time.Second,
		PoolTimeout:  4 * time.Second,
		IdleTimeout:  5 * time.Minute,
	}
}

func (c *Config) Addr() string {
	return fmt.Sprintf("%s:%d", c.Host, c.Port)
}

func (c *Config) Validate() error {
	if c.Host == "" {
		return fmt.Errorf("redis host cannot be empty")
	}
	if c.Port <= 0 || c.Port > 65535 {
		return fmt.Errorf("redis port must be between 1 and 65535")
	}
	if c.PoolSize <= 0 {
		return fmt.Errorf("pool size must be greater than 0")
	}
	if c.MinIdleConns < 0 {
		return fmt.Errorf("min idle connections cannot be negative")
	}
	if c.MinIdleConns > c.PoolSize {
		return fmt.Errorf("min idle connections cannot be greater than pool size")
	}
	if c.DB < 0 {
		return fmt.Errorf("db index cannot be negative")
	}
	return nil
}
