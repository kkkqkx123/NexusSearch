pub mod config;
pub mod error;
pub mod api;

// Re-export core API (always available)
pub use api::core;

// Re-export core types for backward compatibility
pub use api::core::{
    IndexManager, IndexManagerConfig, IndexSchema, LogMergePolicyConfig, MergePolicyType,
    ReloadPolicyConfig, SearchOptions, SearchResult as CoreSearchResult,
};

// Re-export embedded API (with embedded feature)
#[cfg(feature = "embedded")]
pub use api::embedded;

#[cfg(feature = "embedded")]
pub use api::embedded::{Bm25Index, SearchResult};

// Re-export server API (with service feature)
#[cfg(feature = "service")]
pub use api::server;

#[cfg(feature = "service")]
pub use api::server::{
    Config as ServiceConfig, IndexConfig as ServiceIndexConfig, RedisConfig, ServerConfig,
};

#[cfg(feature = "service")]
pub use api::server::{init_logging, init_metrics};

#[cfg(feature = "service")]
pub use api::server::{run_server, BM25Service};

// Re-export error types
pub use error::{Bm25Error, Result};

// Re-export config types
pub use config::{Bm25Config, FieldWeights, SearchConfig};
pub use config::IndexManagerConfigBuilder;
