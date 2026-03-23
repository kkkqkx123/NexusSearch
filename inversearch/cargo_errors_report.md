# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 62
- **Total Issues**: 62
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 47
- **Files with Issues**: 32

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 62

### Warning Type Breakdown

- **warning**: 62 warnings

### Files with Warnings (Top 10)

- `src\compress\cache.rs`: 5 warnings
- `src\storage\redis.rs`: 5 warnings
- `src\storage\wal.rs`: 5 warnings
- `src\serialize\document.rs`: 4 warnings
- `src\compress\lcg.rs`: 3 warnings
- `src\index\mod.rs`: 3 warnings
- `src\index\remover.rs`: 2 warnings
- `src\keystore\mod.rs`: 2 warnings
- `src\search\coordinator.rs`: 2 warnings
- `src\search\single_term.rs`: 2 warnings

## Detailed Warning Categorization

### warning: unused variable: `limit`: help: if this is intentional, prefix it with an underscore: `_limit`

**Total Occurrences**: 62  
**Unique Files**: 32

#### `src\storage\redis.rs`: 5 occurrences

- Line 5: unused import: `AsyncCommands`
- Line 6: unused import: `std::collections::HashMap`
- Line 174: variable does not need to be mutable
- ... 2 more occurrences in this file

#### `src\compress\cache.rs`: 5 occurrences

- Line 88: unused import: `std::sync::Arc`
- Line 72: unnecessary `unsafe` block: unnecessary `unsafe` block
- Line 48: creating a shared reference to mutable static: shared reference to mutable static
- ... 2 more occurrences in this file

#### `src\storage\wal.rs`: 5 occurrences

- Line 10: unused import: `Path`
- Line 174: use of deprecated function `base64::encode`: Use Engine::encode
- Line 211: use of deprecated function `base64::encode`: Use Engine::encode
- ... 2 more occurrences in this file

#### `src\serialize\document.rs`: 4 occurrences

- Line 5: unused imports: `Field` and `TagSystem`
- Line 7: unused import: `crate::r#type::DocId`
- Line 9: unused import: `serde_json::Value`
- ... 1 more occurrences in this file

#### `src\index\mod.rs`: 3 occurrences

- Line 2: unused imports: `KeystoreArray` and `ResolutionSlot`
- Line 3: unused import: `crate::tokenizer::Tokenizer`
- Line 4: unused import: `InversearchError`

#### `src\compress\lcg.rs`: 3 occurrences

- Line 4: constant `UINT32_MAX` is never used
- Line 30: function `lcg_for_u32` is never used
- Line 34: function `lcg_result_to_u32` is never used

#### `src\async_.rs`: 2 occurrences

- Line 5: unused import: `SearchResults`
- Line 238: type `async_::BatchOperation` is more private than the item `async_::BatchAsyncOperations::add_operation`: method `async_::BatchAsyncOperations::add_operation` is reachable at visibility `pub`

#### `src\search\single_term.rs`: 2 occurrences

- Line 8: unused import: `std::collections::HashMap`
- Line 140: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\serialize\types.rs`: 2 occurrences

- Line 5: unused import: `crate::error::Result`
- Line 6: unused import: `DocId`

#### `src\serialize\chunked.rs`: 2 occurrences

- Line 100: variable does not need to be mutable
- Line 129: variable does not need to be mutable

#### `src\compress\radix.rs`: 2 occurrences

- Line 45: function `to_radix_u32` is never used
- Line 49: function `to_radix_usize` is never used

#### `src\search\coordinator.rs`: 2 occurrences

- Line 17: unused import: `Field`
- Line 488: unused import: `crate::SearchOptions`

#### `src\document\mod.rs`: 2 occurrences

- Line 32: unused imports: `IndexOptions` and `Index`
- Line 47: field `fastupdate` is never read

#### `src\index\remover.rs`: 2 occurrences

- Line 2: unused import: `InversearchError`
- Line 3: unused import: `std::collections::HashMap`

#### `src\document\field.rs`: 2 occurrences

- Line 5: unused import: `Encoder`
- Line 102: variable does not need to be mutable

#### `src\resolver\handler.rs`: 2 occurrences

- Line 11: unused variable: `offset`: help: if this is intentional, prefix it with an underscore: `_offset`
- Line 88: variable does not need to be mutable

#### `src\keystore\mod.rs`: 2 occurrences

- Line 349: function `lcg64` is never used
- Line 357: function `lcg_for_number` is never used

#### `src\resolver\and.rs`: 1 occurrences

- Line 4: unused variable: `limit`: help: if this is intentional, prefix it with an underscore: `_limit`

#### `src\storage\mod.rs`: 1 occurrences

- Line 13: unused import: `async_trait::async_trait`

#### `src\search\multi_field.rs`: 1 occurrences

- Line 47: method `set_weight` is never used

#### `src\common\mod.rs`: 1 occurrences

- Line 1: unused import: `crate::encoder::Encoder`

#### `src\intersect\suggestion.rs`: 1 occurrences

- Line 6: unused import: `std::collections::HashMap`

#### `src\resolver\resolver.rs`: 1 occurrences

- Line 466: unused import: `crate::r#type::SearchOptions`

#### `src\resolver\xor.rs`: 1 occurrences

- Line 3: unused variable: `boost`: help: if this is intentional, prefix it with an underscore: `_boost`

#### `src\highlight\boundary.rs`: 1 occurrences

- Line 2: unused import: `crate::highlight::matcher::*`

#### `src\document\batch.rs`: 1 occurrences

- Line 251: field `batch_size` is never read

#### `src\highlight\core.rs`: 1 occurrences

- Line 84: unused variable: `boundary`: help: if this is intentional, prefix it with an underscore: `_boundary`

#### `src\highlight\matcher.rs`: 1 occurrences

- Line 25: unused variable: `doc_org_cur_len`: help: if this is intentional, prefix it with an underscore: `_doc_org_cur_len`

#### `src\search\mod.rs`: 1 occurrences

- Line 56: unused variable: `context`: help: if this is intentional, prefix it with an underscore: `_context`

#### `src\resolver\mod.rs`: 1 occurrences

- Line 50: variable does not need to be mutable

#### `src\serialize\async.rs`: 1 occurrences

- Line 8: unused import: `std::sync::Arc`

#### `src\highlight\processor.rs`: 1 occurrences

- Line 4: unused import: `crate::highlight::core::*`

