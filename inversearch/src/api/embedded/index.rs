use crate::{
    Index,
    index::IndexOptions,
    r#type::SearchOptions,
    search::search,
    keystore::DocId,
    error::Result,
};

/// Embedded search result - simplified for library users
#[derive(Debug, Clone)]
pub struct EmbeddedSearchResult {
    pub id: DocId,
    pub content: String,
    pub score: f32,
    pub highlights: Option<Vec<String>>,
}

/// Embedded index configuration
#[derive(Debug, Clone)]
pub struct EmbeddedIndexConfig {
    pub index_path: String,
    pub enable_highlighting: bool,
    pub default_search_limit: usize,
}

impl Default for EmbeddedIndexConfig {
    fn default() -> Self {
        Self {
            index_path: "./index".to_string(),
            enable_highlighting: true,
            default_search_limit: 10,
        }
    }
}

/// Embedded index statistics
#[derive(Debug, Clone)]
pub struct EmbeddedIndexStats {
    pub document_count: usize,
    pub index_path: String,
}

/// Batch operation type
#[derive(Debug, Clone)]
pub enum EmbeddedBatchOperation {
    Add { id: DocId, content: String },
    Remove { id: DocId },
}

/// Batch operation result
#[derive(Debug, Clone)]
pub struct EmbeddedBatchResult {
    pub success_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

/// Embedded index - high-level API for library users
pub struct EmbeddedIndex {
    index: Index,
    config: EmbeddedIndexConfig,
}

impl EmbeddedIndex {
    /// Create a new index
    pub fn create(_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let config = EmbeddedIndexConfig::default();
        Self::create_with_config(_path, config)
    }

    /// Create a new index with custom configuration
    pub fn create_with_config(
        _path: impl AsRef<std::path::Path>,
        config: EmbeddedIndexConfig,
    ) -> Result<Self> {
        let index_options = IndexOptions::default();
        let index = Index::new(index_options)?;
        Ok(Self { index, config })
    }

    /// Open an existing index
    pub fn open(_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let config = EmbeddedIndexConfig::default();
        Self::open_with_config(_path, config)
    }

    /// Open an existing index with custom configuration
    pub fn open_with_config(
        _path: impl AsRef<std::path::Path>,
        config: EmbeddedIndexConfig,
    ) -> Result<Self> {
        let index_options = IndexOptions::default();
        let index = Index::new(index_options)?;
        Ok(Self { index, config })
    }

    /// Add a document
    pub fn add(&mut self, id: DocId, content: impl Into<String>) -> Result<()> {
        let content = content.into();
        self.index.add(id, &content, false)
    }

    /// Add a document with fields
    pub fn add_with_fields(
        &mut self,
        id: DocId,
        fields: Vec<(String, String)>,
    ) -> Result<()> {
        // Combine fields into content
        let content = fields.into_iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");
        self.index.add(id, &content, false)
    }

    /// Remove a document
    pub fn remove(&mut self, id: DocId) -> Result<()> {
        self.index.remove(id, false)
    }

    /// Search with default limit
    pub fn search(
        &self,
        query: impl Into<String>,
    ) -> Result<Vec<EmbeddedSearchResult>> {
        let limit = self.config.default_search_limit;
        self.search_with_limit(query, limit)
    }

    /// Search with custom limit
    pub fn search_with_limit(
        &self,
        query: impl Into<String>,
        limit: usize,
    ) -> Result<Vec<EmbeddedSearchResult>> {
        let query_str = query.into();
        let search_opts = SearchOptions {
            query: Some(query_str.clone()),
            limit: Some(limit),
            ..Default::default()
        };

        let result = search(&self.index, &search_opts)?;
        
        // Convert to embedded results
        let embedded_results: Vec<EmbeddedSearchResult> = result.results
            .into_iter()
            .map(|doc_id| EmbeddedSearchResult {
                id: doc_id,
                content: String::new(), // TODO: retrieve actual content
                score: 1.0,
                highlights: None,
            })
            .collect();

        Ok(embedded_results)
    }

    /// Get index statistics
    pub fn stats(&self) -> EmbeddedIndexStats {
        EmbeddedIndexStats {
            document_count: self.index.document_count(),
            index_path: self.config.index_path.clone(),
        }
    }

    /// Clear all documents from the index
    pub fn clear(&mut self) {
        self.index.clear();
    }

    /// Create a batch operation builder
    pub fn batch(&mut self) -> EmbeddedBatch<'_> {
        EmbeddedBatch::new(self)
    }

    /// Get the internal index (for advanced usage)
    pub fn inner(&self) -> &Index {
        &self.index
    }

    /// Get the mutable internal index (for advanced usage)
    pub fn inner_mut(&mut self) -> &mut Index {
        &mut self.index
    }
}

/// Batch operation builder
pub struct EmbeddedBatch<'a> {
    index: &'a mut EmbeddedIndex,
    operations: Vec<EmbeddedBatchOperation>,
}

impl<'a> EmbeddedBatch<'a> {
    /// Create a new batch
    pub fn new(index: &'a mut EmbeddedIndex) -> Self {
        Self {
            index,
            operations: Vec::new(),
        }
    }

    /// Add an add operation
    pub fn add(mut self, id: DocId, content: impl Into<String>) -> Self {
        self.operations.push(EmbeddedBatchOperation::Add {
            id,
            content: content.into(),
        });
        self
    }

    /// Add a remove operation
    pub fn remove(mut self, id: DocId) -> Self {
        self.operations.push(EmbeddedBatchOperation::Remove { id });
        self
    }

    /// Execute all operations
    pub fn execute(self) -> EmbeddedBatchResult {
        let mut success_count = 0;
        let mut failed_count = 0;
        let mut errors = Vec::new();

        for op in self.operations {
            match op {
                EmbeddedBatchOperation::Add { id, content } => {
                    match self.index.add(id, &content) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            failed_count += 1;
                            errors.push(format!("Failed to add {}: {}", id, e));
                        }
                    }
                }
                EmbeddedBatchOperation::Remove { id } => {
                    match self.index.remove(id) {
                        Ok(_) => success_count += 1,
                        Err(e) => {
                            failed_count += 1;
                            errors.push(format!("Failed to remove {}: {}", id, e));
                        }
                    }
                }
            }
        }

        EmbeddedBatchResult {
            success_count,
            failed_count,
            errors,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embedded_index_config_default() {
        let config = EmbeddedIndexConfig::default();
        assert_eq!(config.index_path, "./index");
        assert_eq!(config.default_search_limit, 10);
        assert!(config.enable_highlighting);
    }

    #[test]
    fn test_embedded_index_create() {
        let index = EmbeddedIndex::create("./test_index");
        assert!(index.is_ok());
    }

    #[test]
    fn test_embedded_index_add_and_search() {
        let mut index = EmbeddedIndex::create("./test_index").unwrap();
        
        // Add some documents
        assert!(index.add(1, "Hello world").is_ok());
        assert!(index.add(2, "Rust programming").is_ok());
        assert!(index.add(3, "Hello Rust").is_ok());
        
        // Search
        let results = index.search("hello");
        assert!(results.is_ok());
        
        let results = results.unwrap();
        assert!(!results.is_empty());
    }

    #[test]
    fn test_embedded_index_batch() {
        let mut index = EmbeddedIndex::create("./test_index").unwrap();
        
        let result = index.batch()
            .add(1, "Document 1")
            .add(2, "Document 2")
            .add(3, "Document 3")
            .execute();
        
        assert_eq!(result.success_count, 3);
        assert_eq!(result.failed_count, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_embedded_index_remove() {
        let mut index = EmbeddedIndex::create("./test_index").unwrap();
        
        // Add and then remove
        assert!(index.add(1, "Test document").is_ok());
        assert!(index.remove(1).is_ok());
        
        // Search should return no results
        let results = index.search("test").unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_embedded_index_stats() {
        let mut index = EmbeddedIndex::create("./test_index").unwrap();
        
        let stats_before = index.stats();
        assert_eq!(stats_before.document_count, 0);
        
        index.add(1, "Test").unwrap();
        
        let stats_after = index.stats();
        assert_eq!(stats_after.document_count, 1);
    }

    #[test]
    fn test_embedded_index_clear() {
        let mut index = EmbeddedIndex::create("./test_index").unwrap();
        
        index.add(1, "Test 1").unwrap();
        index.add(2, "Test 2").unwrap();
        
        index.clear();
        
        let stats = index.stats();
        assert_eq!(stats.document_count, 0);
    }
}
