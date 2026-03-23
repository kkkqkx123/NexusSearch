# NexusSearch

## Project Architecture

The system is structured in a layered microservices architecture as follows:

- **Client Layer**: Initiates HTTP or gRPC requests to the backend.
- **API Gateway (Go)**: Serves as the entry point, handling authentication/authorization, rate limiting and circuit breaking, request routing, and unified error responses.
- **Query Coordinator (Go)**: Receives routed requests and orchestrates search execution by:
  - Routing queries intelligently,
  - Invoking multiple search engines in parallel,
  - Fusing results using RRF (Reciprocal Rank Fusion) or weighted scoring,
  - Managing query caching via Redis,
  - Enforcing timeouts and fallback mechanisms.
- **Search Engine Services (Rust)**: Three specialized engines communicate with the coordinator via gRPC:
  - *Inversearch*: Keyword-based search with fuzzy matching and phrase queries.
  - *BM25 Service*: Full-text search using the BM25 ranking algorithm.
  - *Vector Search Service*: Semantic search based on vector embeddings and similarity retrieval.

## Directory Structure

```
flexsearch-0.8.2/
├── api-gateway/   # API Gateway (Go)
├── coordinator/   # Query Coordinator (Go)
├── inversearch/   # Keyword search service (Rust)
├── bm25/          # BM25 full-text search service (Rust)
├── shared/        # Shared utilities and libraries (Go)
```

> Note: The `coordinator/` directory appears twice in the original listing — this is likely a duplication; only one instance is needed.

## Core Features

1. **API Gateway**
   - JWT token validation and API key authentication
   - Rate limiting based on user or IP address
   - Request routing and protocol/format translation
   - Centralized error handling

2. **Query Coordinator**
   - Intelligent query routing across search engines
   - Parallel execution of multiple search backends
   - Result fusion using RRF or custom weighting strategies
   - Redis-based query caching
   - Timeout enforcement and graceful degradation

3. **Inversearch Service**
   - High-performance inverted index implementation
   - Support for exact, fuzzy, and phrase matching
   - Result highlighting
   - Search suggestion generation

4. **BM25 Service**
   - Full-text search using the BM25 ranking algorithm
   - Term frequency and document frequency statistics
   - Tunable parameters (k1, b)
   - Multilingual tokenization support

## Technology Stack

### Go Services
- Go 1.23+
- Gin (HTTP framework)
- gRPC (inter-service communication)
- Redis (caching and rate limiting)
- Zap (structured logging)
- Prometheus (metrics and monitoring)
- Viper (configuration management)

### Rust Services
- Rust 2021 edition
- Tokio (async runtime)
- Tonic (gRPC framework)
- Tantivy (search engine library)
- Redis (caching)
- Tracing (logging)
- Metrics (observability)

## Development Conventions

### Code Style
- Go: Follow standard Go coding conventions (e.g., `gofmt`, effective Go guidelines)
- Rust: Adhere to Rust idiomatic practices (e.g., `rustfmt`, clippy lints)

### Testing Practices
- Go: Use the built-in `testing` package with table-driven tests
- Rust: Leverage `cargo test` for unit and integration testing

## Supported Languages / Character Sets
- Latin scripts
- Chinese, Korean, Japanese (CJK)
- Hindi
- Arabic
- Cyrillic
- Greek and Coptic
- Hebrew

## API Overview

### Main Methods
- `add(id, content)` — Add content to the index
- `search(query, options)` — Execute a search with optional parameters (e.g., filters, ranking mode)
- `update(id, content)` — Update existing indexed content
- `remove(id)` — Remove content from the index by ID
- `clear()` — Clear the entire index