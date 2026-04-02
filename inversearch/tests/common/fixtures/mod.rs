//! 测试固件模块
//!
//! 提供测试所需的所有数据固件

pub mod documents;
pub mod queries;
pub mod helpers;

// 重新导出常用固件
pub use documents::*;
pub use queries::*;
pub use helpers::*;
