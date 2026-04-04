pub mod config;
pub mod error;
pub mod index;

#[cfg(feature = "service")]
pub mod service;

pub use config::{Bm25Config, FieldWeights, SearchConfig};
pub use error::{Bm25Error, Result};
pub use index::{Bm25Index, IndexManager, IndexManagerConfig, IndexSchema, SearchResult};

pub use config::IndexManagerConfigBuilder;

#[cfg(feature = "service")]
pub use service::{Config as ServiceConfig, IndexConfig as ServiceIndexConfig, RedisConfig, ServerConfig};

#[cfg(feature = "service")]
pub use service::{init_logging, init_metrics};

#[cfg(feature = "service")]
pub use service::{run_server, BM25Service};
