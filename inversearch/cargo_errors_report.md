# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 149
- **Total Issues**: 149
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 86
- **Files with Issues**: 48

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 149

### Warning Type Breakdown

- **warning**: 149 warnings

### Files with Warnings (Top 10)

- `src\keystore\mod.rs`: 13 warnings
- `tests\common\fixtures\helpers.rs`: 11 warnings
- `tests\common\fixtures\documents.rs`: 9 warnings
- `src\resolver\enrich.rs`: 9 warnings
- `src\index\builder.rs`: 6 warnings
- `src\search\coordinator.rs`: 6 warnings
- `tests\common\fixtures\queries.rs`: 5 warnings
- `src\resolver\resolver.rs`: 5 warnings
- `src\serialize\index.rs`: 5 warnings
- `src\serialize\chunked.rs`: 5 warnings

## Detailed Warning Categorization

### warning: the variable `position` is used as a loop counter: help: consider using: `for (position, token) in tokens.iter().enumerate()`

**Total Occurrences**: 149  
**Unique Files**: 48

#### `src\keystore\mod.rs`: 13 occurrences

- Line 90: you seem to want to iterate on a map's values
- Line 100: you seem to want to iterate on a map's values
- Line 110: you seem to want to iterate on a map's values
- ... 10 more occurrences in this file

#### `tests\common\fixtures\helpers.rs`: 11 occurrences

- Line 9: function `create_index_with_docs` is never used
- Line 20: function `create_empty_index` is never used
- Line 25: function `create_english_index` is never used
- ... 8 more occurrences in this file

#### `src\resolver\enrich.rs`: 9 occurrences

- Line 152: you should consider adding a `Default` implementation for `Enricher`
- Line 183: writing `&Vec` instead of `&[_]` involves a new object where a slice will do: help: change this to: `&[Option<Value>]`
- Line 205: writing `&Vec` instead of `&[_]` involves a new object where a slice will do: help: change this to: `&[Option<Value>]`
- ... 6 more occurrences in this file

#### `tests\common\fixtures\documents.rs`: 9 occurrences

- Line 7: struct `TestDocument` is never constructed
- Line 14: constant `PROGRAMMING_DOCS` is never used
- Line 58: constant `CHINESE_DOCS` is never used
- ... 6 more occurrences in this file

#### `src\search\coordinator.rs`: 6 occurrences

- Line 61: this `impl` can be derived
- Line 287: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `field.index()`
- Line 354: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `field.index()`
- ... 3 more occurrences in this file

#### `src\index\builder.rs`: 6 occurrences

- Line 14: this `if` statement can be collapsed
- Line 131: this function has too many arguments (11/7)
- Line 173: this function has too many arguments (10/7)
- ... 3 more occurrences in this file

#### `tests\common\fixtures\queries.rs`: 5 occurrences

- Line 7: struct `TestQuery` is never constructed
- Line 14: constant `BASIC_QUERIES` is never used
- Line 38: constant `CJK_QUERIES` is never used
- ... 2 more occurrences in this file

#### `src\serialize\chunked.rs`: 5 occurrences

- Line 69: you seem to want to iterate on a map's keys
- Line 76: manually reimplementing `div_ceil`: help: consider using `.div_ceil()`: `items.len().div_ceil(chunk_size)`
- Line 105: manually reimplementing `div_ceil`: help: consider using `.div_ceil()`: `items.len().div_ceil(chunk_size)`
- ... 2 more occurrences in this file

#### `src\config\mod.rs`: 5 occurrences

- Line 180: this `impl` can be derived
- Line 275: used `assert_eq!` with a literal bool
- Line 282: used `assert_eq!` with a literal bool
- ... 2 more occurrences in this file

#### `src\resolver\resolver.rs`: 5 occurrences

- Line 40: this `impl` can be derived
- Line 141: this `impl` can be derived
- Line 175: this pattern creates a reference to a reference: help: try: `query`
- ... 2 more occurrences in this file

#### `src\intersect\suggestion.rs`: 5 occurrences

- Line 35: this `impl` can be derived
- Line 65: field assignment outside of initializer for an instance created with Default::default()
- Line 136: you seem to use `.enumerate()` and immediately discard the index
- ... 2 more occurrences in this file

#### `src\serialize\index.rs`: 5 occurrences

- Line 55: you seem to want to iterate on a map's values
- Line 68: you seem to want to iterate on a map's values
- Line 85: you seem to want to iterate on a map's values
- ... 2 more occurrences in this file

#### `src\index\mod.rs`: 4 occurrences

- Line 169: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 167: use of `or_insert_with` to construct default value: help: try: `or_default()`
- Line 194: use of `or_insert_with` to construct default value: help: try: `or_default()`
- ... 1 more occurrences in this file

#### `src\document\tree.rs`: 4 occurrences

- Line 66: this `impl` can be derived
- Line 158: stripping a prefix manually
- Line 192: stripping a prefix manually
- ... 1 more occurrences in this file

#### `tests\common\fixtures\mod.rs`: 3 occurrences

- Line 10: unused import: `documents::*`
- Line 11: unused import: `queries::*`
- Line 12: unused import: `helpers::*`

#### `src\encoder\mod.rs`: 3 occurrences

- Line 704: found call to `str::trim` before `str::split_whitespace`: help: remove `trim()`
- Line 782: field assignment outside of initializer for an instance created with Default::default()
- Line 821: field assignment outside of initializer for an instance created with Default::default()

#### `src\storage\wal.rs`: 3 occurrences

- Line 196: manual implementation of `.is_multiple_of()`: help: replace with: `self.change_count.load(Ordering::Relaxed).is_multiple_of(self.config.snapshot_interval)`
- Line 369: unnecessary `if let` since only the `Ok` variant of the iterator element is used
- Line 505: variable does not need to be mutable

#### `src\service.rs`: 3 occurrences

- Line 279: field assignment outside of initializer for an instance created with Default::default()
- Line 293: casting to the same type is unnecessary (`u64` -> `u64`): help: try: `id`
- Line 400: unused variable: `service`: help: if this is intentional, prefix it with an underscore: `_service`

#### `src\charset\latin\mod.rs`: 3 occurrences

- Line 70: manual case-insensitive ASCII comparison
- Line 70: manual case-insensitive ASCII comparison
- Line 225: items after a test module

#### `src\resolver\mod.rs`: 3 occurrences

- Line 17: module has the same name as its containing module
- Line 50: variable does not need to be mutable
- Line 79: field assignment outside of initializer for an instance created with Default::default()

#### `src\document\tag.rs`: 3 occurrences

- Line 27: very complex type used. Consider factoring parts into `type` definitions
- Line 58: you should consider adding a `Default` implementation for `TagSystem`
- Line 74: very complex type used. Consider factoring parts into `type` definitions

#### `src\search\single_term.rs`: 2 occurrences

- Line 174: function `perform_intersection` is never used
- Line 19: this function has too many arguments (8/7)

#### `src\intersect\core.rs`: 2 occurrences

- Line 73: usage of `contains_key` followed by `insert` on a `HashMap`
- Line 180: usage of `contains_key` followed by `insert` on a `HashMap`

#### `src\intersect\scoring.rs`: 2 occurrences

- Line 159: you should consider adding a `Default` implementation for `ScoreManager`
- Line 209: you seem to use `.enumerate()` and immediately discard the index

#### `src\serialize\document.rs`: 2 occurrences

- Line 65: you seem to want to iterate on a map's values
- Line 133: casting to the same type is unnecessary (`u64` -> `u64`): help: try: `data.registry.next_doc_id`

#### `src\search\multi_field.rs`: 2 occurrences

- Line 99: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `field.index()`
- Line 109: unnecessary map of the identity function: help: remove the call to `map`

#### `src\document\batch.rs`: 2 occurrences

- Line 277: the following explicit lifetimes could be elided: 'a
- Line 308: the following explicit lifetimes could be elided: 'a

#### `src\highlight\core.rs`: 2 occurrences

- Line 147: casting to the same type is unnecessary (`i32` -> `i32`): help: try: `boundary_before`
- Line 153: casting to the same type is unnecessary (`i32` -> `i32`): help: try: `boundary_after`

#### `src\resolver\async_resolver.rs`: 2 occurrences

- Line 84: method `borrow` can be confused for the standard trait method `std::borrow::Borrow::borrow`
- Line 88: method `borrow_mut` can be confused for the standard trait method `std::borrow::BorrowMut::borrow_mut`

#### `src\index\remover.rs`: 2 occurrences

- Line 106: using `clone` on type `usize` which implements the `Copy` trait: help: try dereferencing it: `*term_hash`
- Line 124: using `clone` on type `usize` which implements the `Copy` trait: help: try dereferencing it: `*term_hash`

#### `src\tokenizer\mod.rs`: 1 occurrences

- Line 107: the variable `position` is used as a loop counter: help: consider using: `for (position, token) in tokens.iter().enumerate()`

#### `src\highlight\types.rs`: 1 occurrences

- Line 113: you should consider adding a `Default` implementation for `EncoderCache`

#### `src\serialize\async.rs`: 1 occurrences

- Line 63: this `if` has identical blocks

#### `tests\search\edge_case_test.rs`: 1 occurrences

- Line 77: unused variable: `result`: help: if this is intentional, prefix it with an underscore: `_result`

#### `tests\common\mod.rs`: 1 occurrences

- Line 8: unused import: `fixtures::*`

#### `src\main.rs`: 1 occurrences

- Line 49: this assertion is always `true`

#### `src\document\field.rs`: 1 occurrences

- Line 28: very complex type used. Consider factoring parts into `type` definitions

#### `src\resolver\not.rs`: 1 occurrences

- Line 11: this `if` statement can be collapsed

#### `src\storage\redis.rs`: 1 occurrences

- Line 472: you should consider adding a `Default` implementation for `StorageMetrics`

#### `src\document\mod.rs`: 1 occurrences

- Line 342: very complex type used. Consider factoring parts into `type` definitions

#### `src\serialize\format.rs`: 1 occurrences

- Line 28: redundant slicing of the whole range: help: use the original value instead: `bytes`

#### `src\resolver\or.rs`: 1 occurrences

- Line 20: usage of `contains_key` followed by `insert` on a `HashMap`

#### `tests\service\grpc_test.rs`: 1 occurrences

- Line 177: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`: help: try: `if let Ok(_) = response {}`

#### `src\async_.rs`: 1 occurrences

- Line 358: field assignment outside of initializer for an instance created with Default::default()

#### `src\highlight\tests.rs`: 1 occurrences

- Line 22: used `assert_eq!` with a literal bool

#### `src\resolver\handler.rs`: 1 occurrences

- Line 395: field assignment outside of initializer for an instance created with Default::default()

#### `src\encoder\validator.rs`: 1 occurrences

- Line 132: field assignment outside of initializer for an instance created with Default::default()

#### `src\search\cache.rs`: 1 occurrences

- Line 351: field assignment outside of initializer for an instance created with Default::default()

