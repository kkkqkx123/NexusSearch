//! 存储模块共享类型
//!
//! 定义各存储实现共享的数据结构和类型

use serde::{Serialize, Deserialize};

/// 存储信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub name: String,
    pub version: String,
    pub size: u64,
    pub document_count: usize,
    pub index_count: usize,
    pub is_connected: bool,
}

/// 存储性能指标
#[derive(Debug, Clone)]
pub struct StorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub error_count: usize,
}

impl Default for StorageMetrics {
    fn default() -> Self {
        Self {
            operation_count: 0,
            average_latency: 0,
            memory_usage: 0,
            error_count: 0,
        }
    }
}

impl StorageMetrics {
    /// 创建空的指标
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置所有指标
    pub fn reset(&mut self) {
        self.operation_count = 0;
        self.average_latency = 0;
        self.memory_usage = 0;
        self.error_count = 0;
    }
}

#[cfg(feature = "store-memory")]
#[derive(Debug, Clone)]
pub struct MemoryStorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub error_count: usize,
}

/// 文件存储数据格式
/// 
/// 用于文件存储和缓存存储的序列化格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStorageData {
    pub version: String,
    pub timestamp: String,
    pub data: std::collections::HashMap<String, Vec<crate::r#type::DocId>>,
    pub context_data: std::collections::HashMap<String, std::collections::HashMap<String, Vec<crate::r#type::DocId>>>,
    pub documents: std::collections::HashMap<crate::r#type::DocId, String>,
}

#[cfg(feature = "store-file")]
#[derive(Debug, Clone)]
pub struct FileStorageMetrics {
    pub operation_count: usize,
    pub average_latency: usize,
    pub memory_usage: usize,
    pub file_size: u64,
    pub error_count: usize,
}
