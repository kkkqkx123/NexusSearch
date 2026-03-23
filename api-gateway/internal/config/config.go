package config

import (
	"fmt"
	"time"

	"github.com/spf13/viper"
)

type Config struct {
	Server      ServerConfig      `mapstructure:"server"`
	Log         LogConfig         `mapstructure:"log"`
	Redis       RedisConfig       `mapstructure:"redis"`
	Coordinator CoordinatorConfig `mapstructure:"coordinator"`
	JWT         JWTConfig         `mapstructure:"jwt"`
	RateLimit   RateLimitConfig   `mapstructure:"ratelimit"`
	CORS        CORSConfig        `mapstructure:"cors"`
}

type ServerConfig struct {
	Port         int    `mapstructure:"port"`
	Mode         string `mapstructure:"mode"`
	ReadTimeout  int    `mapstructure:"read_timeout"`
	WriteTimeout int    `mapstructure:"write_timeout"`
}

type LogConfig struct {
	Level  string `mapstructure:"level"`
	Format string `mapstructure:"format"`
	Output string `mapstructure:"output"`
}

type RedisConfig struct {
	Host         string `mapstructure:"host"`
	Port         int    `mapstructure:"port"`
	Password     string `mapstructure:"password"`
	DB           int    `mapstructure:"db"`
	PoolSize     int    `mapstructure:"pool_size"`
	MinIdleConns int    `mapstructure:"min_idle_conns"`
	MaxRetries   int    `mapstructure:"max_retries"`
	DialTimeout  int    `mapstructure:"dial_timeout"`
	ReadTimeout  int    `mapstructure:"read_timeout"`
	WriteTimeout int    `mapstructure:"write_timeout"`
	PoolTimeout  int    `mapstructure:"pool_timeout"`
	IdleTimeout  int    `mapstructure:"idle_timeout"`
}

type CoordinatorConfig struct {
	Address string `mapstructure:"address"`
	Timeout int    `mapstructure:"timeout"`
}

type JWTConfig struct {
	Secret     string `mapstructure:"secret"`
	Expiration int    `mapstructure:"expiration"`
	Issuer     string `mapstructure:"issuer"`
}

type RateLimitConfig struct {
	Enabled       bool          `mapstructure:"enabled"`
	DefaultLimit  int           `mapstructure:"default_limit"`
	DefaultWindow time.Duration `mapstructure:"default_window"`
	ByUser        bool          `mapstructure:"by_user"`
	ByIP          bool          `mapstructure:"by_ip"`
}

type CORSConfig struct {
	Enabled          bool     `mapstructure:"enabled"`
	AllowOrigins     []string `mapstructure:"allow_origins"`
	AllowMethods     []string `mapstructure:"allow_methods"`
	AllowHeaders     []string `mapstructure:"allow_headers"`
	AllowCredentials bool     `mapstructure:"allow_credentials"`
}

func Load() (*Config, error) {
	viper.SetConfigName("config")
	viper.SetConfigType("yaml")
	viper.AddConfigPath("./configs")
	viper.AddConfigPath(".")

	viper.SetEnvPrefix("API_GATEWAY")
	viper.AutomaticEnv()

	if err := viper.ReadInConfig(); err != nil {
		return nil, fmt.Errorf("failed to read config file: %w", err)
	}

	var cfg Config
	if err := viper.Unmarshal(&cfg); err != nil {
		return nil, fmt.Errorf("failed to unmarshal config: %w", err)
	}

	return &cfg, nil
}
