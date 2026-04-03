# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 66
- **Total Issues**: 66
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 43
- **Files with Issues**: 26

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 66

### Warning Type Breakdown

- **warning**: 66 warnings

### Files with Warnings (Top 10)

- `src\keystore\mod.rs`: 10 warnings
- `tests\common\fixtures\documents.rs`: 6 warnings
- `tests\common\fixtures\helpers.rs`: 6 warnings
- `src\intersect\suggestion.rs`: 5 warnings
- `src\index\builder.rs`: 5 warnings
- `src\serialize\index.rs`: 5 warnings
- `src\search\coordinator.rs`: 3 warnings
- `src\resolver\resolver.rs`: 3 warnings
- `src\resolver\async_resolver.rs`: 2 warnings
- `src\service.rs`: 2 warnings

## Detailed Warning Categorization

### warning: this `if` statement can be collapsed

**Total Occurrences**: 66  
**Unique Files**: 26

#### `src\keystore\mod.rs`: 10 occurrences

- Line 343: used `assert_eq!` with a literal bool
- Line 344: used `assert_eq!` with a literal bool
- Line 347: used `assert_eq!` with a literal bool
- ... 7 more occurrences in this file

#### `tests\common\fixtures\documents.rs`: 6 occurrences

- Line 7: struct `TestDocument` is never constructed
- Line 13: constant `PROGRAMMING_DOCS` is never used
- Line 49: constant `CHINESE_DOCS` is never used
- ... 3 more occurrences in this file

#### `tests\common\fixtures\helpers.rs`: 6 occurrences

- Line 9: function `create_index_with_docs` is never used
- Line 25: function `create_english_index` is never used
- Line 30: function `create_full_index` is never used
- ... 3 more occurrences in this file

#### `src\intersect\suggestion.rs`: 5 occurrences

- Line 35: this `impl` can be derived
- Line 65: field assignment outside of initializer for an instance created with Default::default()
- Line 136: you seem to use `.enumerate()` and immediately discard the index
- ... 2 more occurrences in this file

#### `src\index\builder.rs`: 5 occurrences

- Line 129: this function has too many arguments (11/7)
- Line 171: this function has too many arguments (10/7)
- Line 216: redundant slicing of the whole range: help: use the original value instead: `keyword`
- ... 2 more occurrences in this file

#### `src\serialize\index.rs`: 5 occurrences

- Line 55: you seem to want to iterate on a map's values
- Line 68: you seem to want to iterate on a map's values
- Line 85: you seem to want to iterate on a map's values
- ... 2 more occurrences in this file

#### `src\resolver\resolver.rs`: 3 occurrences

- Line 143: this pattern creates a reference to a reference: help: try: `query`
- Line 269: usage of `contains_key` followed by `insert` on a `HashMap`
- Line 335: this `if` statement can be collapsed

#### `src\search\coordinator.rs`: 3 occurrences

- Line 358: unnecessary map of the identity function: help: remove the call to `map`
- Line 358: explicit call to `.into_iter()` in function argument accepting `IntoIterator`
- Line 470: casting to the same type is unnecessary (`f32` -> `f32`): help: try: `count`

#### `src\resolver\async_resolver.rs`: 2 occurrences

- Line 93: unexpected `cfg` condition value: `async`
- Line 1: unused import: `ResolverError`

#### `src\config\mod.rs`: 2 occurrences

- Line 287: used `assert_eq!` with a literal bool
- Line 297: used `assert_eq!` with a literal bool

#### `src\service.rs`: 2 occurrences

- Line 111: unreachable pattern: no value can reach this
- Line 291: called `iter().copied().collect()` on a slice to create a `Vec`. Calling `to_vec()` is both faster and more readable: help: try: `.to_vec()`

#### `tests\common\fixtures\mod.rs`: 2 occurrences

- Line 9: unused import: `documents::*`
- Line 10: unused import: `helpers::*`

#### `src\encoder\mod.rs`: 2 occurrences

- Line 781: field assignment outside of initializer for an instance created with Default::default()
- Line 820: field assignment outside of initializer for an instance created with Default::default()

#### `src\resolver\not.rs`: 1 occurrences

- Line 11: this `if` statement can be collapsed

#### `src\resolver\mod.rs`: 1 occurrences

- Line 17: module has the same name as its containing module

#### `tests\common\mod.rs`: 1 occurrences

- Line 8: unused import: `fixtures::*`

#### `src\storage\cached.rs`: 1 occurrences

- Line 58: field assignment outside of initializer for an instance created with Default::default()

#### `src\resolver\enrich.rs`: 1 occurrences

- Line 219: this `if let` can be collapsed into the outer `if let`

#### `src\highlight\tests.rs`: 1 occurrences

- Line 22: used `assert_eq!` with a literal bool

#### `src\storage\types.rs`: 1 occurrences

- Line 27: this `impl` can be derived

#### `src\storage\wal.rs`: 1 occurrences

- Line 369: `flatten()` will run forever if the iterator repeatedly produces an `Err`: help: replace with: `map_while(Result::ok)`

#### `src\charset\latin\mod.rs`: 1 occurrences

- Line 226: items after a test module

#### `src\serialize\chunked.rs`: 1 occurrences

- Line 221: method `next` can be confused for the standard trait method `std::iter::Iterator::next`

#### `src\encoder\validator.rs`: 1 occurrences

- Line 132: field assignment outside of initializer for an instance created with Default::default()

#### `src\storage\redis.rs`: 1 occurrences

- Line 471: this `impl` can be derived

#### `src\document\tree.rs`: 1 occurrences

- Line 339: useless use of `format!`: help: consider using `.to_string()`: `"Cannot apply wildcard to non-object type".to_string()`

