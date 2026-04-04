//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现
//!
//! ## 模块结构
//!
//! ```text
//! storage/
//! ├── common/              # 公共组件（类型、trait、工具函数）
//! │   ├── types.rs         # 共享类型定义
//! │   ├── trait.rs         # 存储接口 trait
//! │   ├── io.rs            # 文件 I/O 操作
//! │   ├── compression.rs   # 压缩/解压缩
//! │   └── metrics.rs       # 性能指标
//! ├── base.rs              # 存储基类
//! ├── utils.rs             # 工具函数
//! ├── file.rs              # 文件存储实现
//! ├── redis.rs             # Redis 存储实现
//! ├── wal.rs               # WAL 模块
//! ├── wal_storage.rs       # WAL 存储实现
//! ├── cold_warm_cache/     # 冷热缓存存储实现（默认）
//! │   ├── mod.rs
//! │   ├── config.rs
//! │   ├── manager.rs
//! │   ├── policy.rs
//! │   ├── stats.rs
//! │   └── background.rs
//! └── memory.rs            # 内存存储实现（仅用于测试）
//! ```
//!
//! ## 条件编译特性
//!
//! - `store-cold-warm-cache`: 冷热缓存存储（默认启用）
//! - `store-file`: 文件存储
//! - `store-redis`: Redis 存储
//! - `store-wal`: WAL 预写日志存储

// 公共组件 - 所有存储实现共享
pub mod common;

// 存储基类
pub mod base;

// 工具函数
pub mod utils;

// 条件编译的存储实现

#[cfg(feature = "store-file")]
pub mod file;

#[cfg(feature = "store-redis")]
pub mod redis;

#[cfg(feature = "store-wal")]
pub mod wal;

#[cfg(feature = "store-wal")]
pub mod wal_storage;

// 冷热缓存存储实现（默认）
pub mod cold_warm_cache;

// 测试用内存存储（仅用于测试）
#[cfg(test)]
mod memory;

// 重新导出常用类型和 trait，方便使用
pub use common::{
    compression::{compress_data, decompress_data},
    io::{atomic_write, get_file_size, load_from_file, remove_file_safe, save_to_file},
    metrics::{MetricsCollector, OperationTimer},
    FileStorageData, StorageInfo, StorageInterface, StorageMetrics,
};
