# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 35
- **Total Warnings**: 19
- **Total Issues**: 54
- **Unique Error Patterns**: 25
- **Unique Warning Patterns**: 19
- **Files with Issues**: 5

## Error Statistics

**Total Errors**: 35

### Error Type Breakdown

- **error[E0252]**: 9 errors
- **error[E0425]**: 6 errors
- **error[E0433]**: 6 errors
- **error[E0422]**: 4 errors
- **error[E0432]**: 4 errors
- **error[E0407]**: 3 errors
- **error[E0603]**: 3 errors

### Files with Errors (Top 10)

- `src\api\server\grpc.rs`: 23 errors
- `src\lib.rs`: 9 errors
- `src\api\embedded\index.rs`: 1 errors
- `src\service.rs`: 1 errors
- `src\api\core\mod.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 19

### Warning Type Breakdown

- **warning**: 19 warnings

### Files with Warnings (Top 10)

- `src\api\core\mod.rs`: 13 warnings
- `src\lib.rs`: 5 warnings
- `src\service.rs`: 1 warnings

## Detailed Error Categorization

### error[E0252]: the name `FileStorage` is defined multiple times: `FileStorage` reimported here

**Total Occurrences**: 9  
**Unique Files**: 2

#### `src\lib.rs`: 8 occurrences

- Line 139: the name `FileStorage` is defined multiple times: `FileStorage` reimported here
- Line 142: the name `WALStorage` is defined multiple times: `WALStorage` reimported here
- Line 145: the name `WALManager` is defined multiple times: `WALManager` reimported here
- ... 5 more occurrences in this file

#### `src\service.rs`: 1 occurrences

- Line 41: the name `StorageBackend` is defined multiple times: `StorageBackend` reimported here

### error[E0425]: cannot find type `BatchOperationRequest` in this scope: not found in this scope

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\api\server\grpc.rs`: 6 occurrences

- Line 292: cannot find type `BatchOperationRequest` in this scope: not found in this scope
- Line 293: cannot find type `BatchOperationResponse` in this scope: not found in this scope
- Line 402: cannot find type `SuggestRequest` in this scope: not found in this scope
- ... 3 more occurrences in this file

### error[E0433]: failed to resolve: could not find `storage` in `core`: could not find `storage` in `core`

**Total Occurrences**: 6  
**Unique Files**: 1

#### `src\api\server\grpc.rs`: 6 occurrences

- Line 19: failed to resolve: could not find `storage` in `core`: could not find `storage` in `core`
- Line 22: failed to resolve: could not find `storage` in `core`: could not find `storage` in `core`
- Line 25: failed to resolve: could not find `storage` in `core`: could not find `storage` in `core`
- ... 3 more occurrences in this file

### error[E0422]: cannot find struct, variant or union type `BatchOperationResponse` in this scope: not found in this scope

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\api\server\grpc.rs`: 4 occurrences

- Line 350: cannot find struct, variant or union type `BatchOperationResponse` in this scope: not found in this scope
- Line 426: cannot find struct, variant or union type `SuggestResponse` in this scope: not found in this scope
- Line 431: cannot find struct, variant or union type `SuggestResponse` in this scope: not found in this scope
- ... 1 more occurrences in this file

### error[E0432]: unresolved import `crate::api::core::index::IndexOptions`: no `IndexOptions` in `serialize::index`

**Total Occurrences**: 4  
**Unique Files**: 1

#### `src\api\server\grpc.rs`: 4 occurrences

- Line 47: unresolved import `crate::api::core::index::IndexOptions`: no `IndexOptions` in `serialize::index`
- Line 34: unresolved import `crate::api::core::config`: could not find `config` in `core`
- Line 35: unresolved import `crate::api::core::config`: could not find `config` in `core`
- ... 1 more occurrences in this file

### error[E0603]: struct import `SearchOptions` is private: private struct import

**Total Occurrences**: 3  
**Unique Files**: 3

#### `src\api\embedded\index.rs`: 1 occurrences

- Line 4: struct import `SearchOptions` is private: private struct import

#### `src\lib.rs`: 1 occurrences

- Line 35: struct import `SearchOptions` is private: private struct import

#### `src\api\core\mod.rs`: 1 occurrences

- Line 22: struct import `SearchOptions` is private: private struct import

### error[E0407]: method `batch_operation` is not a member of trait `InversearchServiceTrait`: not a member of trait `InversearchServiceTrait`

**Total Occurrences**: 3  
**Unique Files**: 1

#### `src\api\server\grpc.rs`: 3 occurrences

- Line 290: method `batch_operation` is not a member of trait `InversearchServiceTrait`: not a member of trait `InversearchServiceTrait`
- Line 400: method `suggest` is not a member of trait `InversearchServiceTrait`: not a member of trait `InversearchServiceTrait`
- Line 467: method `health_check` is not a member of trait `InversearchServiceTrait`: not a member of trait `InversearchServiceTrait`

## Detailed Warning Categorization

### warning: ambiguous glob re-exports: the name `BatchOperation` in the type namespace is first re-exported here

**Total Occurrences**: 19  
**Unique Files**: 3

#### `src\api\core\mod.rs`: 13 occurrences

- Line 5: ambiguous glob re-exports: the name `BatchOperation` in the type namespace is first re-exported here
- Line 8: ambiguous glob re-exports: the name `CacheStats` in the type namespace is first re-exported here
- Line 11: ambiguous glob re-exports: the name `Tokenizer` in the type namespace is first re-exported here
- ... 10 more occurrences in this file

#### `src\lib.rs`: 5 occurrences

- Line 33: unused imports: `Document`, `FieldType`, `Field`, and `IndexOptions`
- Line 139: unused import: `storage::file::FileStorage`
- Line 142: unused import: `storage::wal_storage::WALStorage`
- ... 2 more occurrences in this file

#### `src\service.rs`: 1 occurrences

- Line 41: unused import: `crate::config::StorageBackend`

