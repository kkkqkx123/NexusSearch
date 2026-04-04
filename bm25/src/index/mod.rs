mod api;
mod batch;
mod delete;
mod document;
pub mod manager;
mod persistence;
mod schema;
mod search;
mod stats;

#[cfg(test)]
mod tests;

pub use api::{Bm25Index, SearchResult};
pub use manager::{
    IndexManager, IndexManagerConfig, LogMergePolicyConfig, MergePolicyType, ReloadPolicyConfig,
};
pub use schema::IndexSchema;
