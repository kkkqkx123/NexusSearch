package config

import (
	"fmt"
	"time"

	"github.com/spf13/viper"
)

type Config struct {
	Server   ServerConfig   `mapstructure:"server"`
	GRPC     GRPCConfig     `mapstructure:"grpc"`
	Redis    RedisConfig    `mapstructure:"redis"`
	Engines  EnginesConfig  `mapstructure:"engines"`
	Cache    CacheConfig    `mapstructure:"cache"`
	Metrics  MetricsConfig  `mapstructure:"metrics"`
	Tracing  TracingConfig  `mapstructure:"tracing"`
	Logging  LoggingConfig  `mapstructure:"logging"`
}

type ServerConfig struct {
	Host            string        `mapstructure:"host"`
	Port            int           `mapstructure:"port"`
	ShutdownTimeout time.Duration `mapstructure:"shutdown_timeout"`
}

type GRPCConfig struct {
	Host            string        `mapstructure:"host"`
	Port            int           `mapstructure:"port"`
	MaxRecvMsgSize  int           `mapstructure:"max_recv_msg_size"`
	MaxSendMsgSize  int           `mapstructure:"max_send_msg_size"`
	Timeout         time.Duration `mapstructure:"timeout"`
}

type CacheConfig struct {
	Enabled         bool          `mapstructure:"enabled"`
	DefaultTTL      time.Duration `mapstructure:"default_ttl"`
	MaxSize         int64         `mapstructure:"max_size"`
	EvictionPolicy  string        `mapstructure:"eviction_policy"`
}

type RedisConfig struct {
	Host     string `mapstructure:"host"`
	Port     int    `mapstructure:"port"`
	Password string `mapstructure:"password"`
	DB       int    `mapstructure:"db"`
	PoolSize int    `mapstructure:"pool_size"`
}

type MetricsConfig struct {
	Enabled bool   `mapstructure:"enabled"`
	Path    string `mapstructure:"path"`
	Port    int    `mapstructure:"port"`
}

type TracingConfig struct {
	Enabled  bool   `mapstructure:"enabled"`
	Exporter string `mapstructure:"exporter"`
	SampleRate float64 `mapstructure:"sample_rate"`
}

type LoggingConfig struct {
	Level      string `mapstructure:"level"`
	Format     string `mapstructure:"format"`
	Output     string `mapstructure:"output"`
}

func Load(configPath string) (*Config, error) {
	v := viper.New()
	v.SetConfigFile(configPath)
	v.SetConfigType("yaml")

	v.SetDefault("server.host", "0.0.0.0")
	v.SetDefault("server.port", 50051)
	v.SetDefault("server.shutdown_timeout", 30*time.Second)

	v.SetDefault("grpc.host", "0.0.0.0")
	v.SetDefault("grpc.port", 50052)
	v.SetDefault("grpc.max_recv_msg_size", 1024*1024*100)
	v.SetDefault("grpc.max_send_msg_size", 1024*1024*100)
	v.SetDefault("grpc.timeout", 30*time.Second)

	v.SetDefault("redis.host", "localhost")
	v.SetDefault("redis.port", 6379)
	v.SetDefault("redis.db", 0)
	v.SetDefault("redis.pool_size", 10)

	v.SetDefault("cache.enabled", true)
	v.SetDefault("cache.default_ttl", 5*time.Minute)
	v.SetDefault("cache.max_size", 10000)
	v.SetDefault("cache.eviction_policy", "lru")

	v.SetDefault("metrics.enabled", true)
	v.SetDefault("metrics.path", "/metrics")
	v.SetDefault("metrics.port", 9090)

	v.SetDefault("tracing.enabled", false)
	v.SetDefault("tracing.exporter", "stdout")
	v.SetDefault("tracing.sample_rate", 1.0)

	v.SetDefault("logging.level", "info")
	v.SetDefault("logging.format", "json")
	v.SetDefault("logging.output", "stdout")

	if err := v.ReadInConfig(); err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	var cfg Config
	if err := v.Unmarshal(&cfg); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config: %w", err)
	}

	return &cfg, nil
}

func (c *Config) GetGRPCAddress() string {
	return fmt.Sprintf("%s:%d", c.GRPC.Host, c.GRPC.Port)
}

func (c *Config) GetMetricsAddress() string {
	return fmt.Sprintf("%s:%d", c.Server.Host, c.Metrics.Port)
}

func (c *Config) GetRedisAddress() string {
	return fmt.Sprintf("%s:%d", c.Redis.Host, c.Redis.Port)
}
