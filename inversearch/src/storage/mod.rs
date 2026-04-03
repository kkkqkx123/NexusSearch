//! 存储接口模块
//!
//! 提供持久化存储的抽象接口和实现
//!
//! ## 条件编译特性
//!
//! - `store-memory`: 内存存储
//! - `store-file`: 文件存储（默认启用）
//! - `store-redis`: Redis 存储
//! - `store-wal`: WAL 预写日志存储
//! - `store-cached`: 缓存存储（内存+文件，默认启用）

pub mod interface;
pub mod types;
pub mod utils;

#[cfg(feature = "store-memory")]
pub mod memory;

#[cfg(feature = "store-file")]
pub mod file;

#[cfg(feature = "store-redis")]
pub mod redis;

#[cfg(feature = "store-wal")]
pub mod wal;

#[cfg(feature = "store-wal")]
pub mod wal_storage;

#[cfg(feature = "store-cached")]
pub mod cached;
