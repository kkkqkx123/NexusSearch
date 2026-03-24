// Service module - only compiled with "service" feature

pub mod config;
pub mod metrics;
pub mod proto;
pub mod grpc;
pub mod cache;

// Re-export service API
pub use config::{Config, ServerConfig, RedisConfig, IndexConfig, CacheConfig};
pub use grpc::{BM25Service, run_server};
pub use metrics::{init_logging, init_metrics};
