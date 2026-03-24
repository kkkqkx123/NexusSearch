// 核心模块（始终编译）
pub mod config;
pub mod error;
pub mod index;

// 功能模块（条件编译）
#[cfg(feature = "service")]
pub mod metrics;

#[cfg(feature = "service")]
pub mod proto;

#[cfg(feature = "service")]
pub mod service;

// 导出核心API（库模式和服务模式都可使用）
pub use config::{Bm25Config, FieldWeights, SearchConfig};
pub use error::{Bm25Error, Result};
pub use index::{IndexManager, IndexSchema};

// 导出服务API（仅服务模式可用）
#[cfg(feature = "service")]
pub use config::Config;

#[cfg(feature = "service")]
pub use metrics::{init_logging, init_metrics};

#[cfg(feature = "service")]
pub use service::{BM25Service, run_server};
