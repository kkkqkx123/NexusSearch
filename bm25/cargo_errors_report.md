# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 3
- **Total Issues**: 3
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 2
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 3

### Warning Type Breakdown

- **warning**: 3 warnings

### Files with Warnings (Top 10)

- `src\api\core\index.rs`: 2 warnings
- `src\api\embedded\index.rs`: 1 warnings

## Detailed Warning Categorization

### warning: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `self.manager.index()`

**Total Occurrences**: 3  
**Unique Files**: 2

#### `src\api\core\index.rs`: 2 occurrences

- Line 18: this `impl` can be derived
- Line 43: this `impl` can be derived

#### `src\api\embedded\index.rs`: 1 occurrences

- Line 103: this expression creates a reference which is immediately dereferenced by the compiler: help: change this to: `self.manager.index()`

