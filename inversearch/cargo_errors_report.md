# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 0
- **Total Warnings**: 6
- **Total Issues**: 6
- **Unique Error Patterns**: 0
- **Unique Warning Patterns**: 6
- **Files with Issues**: 4

## Error Statistics

**Total Errors**: 0

## Warning Statistics

**Total Warnings**: 6

### Warning Type Breakdown

- **warning**: 6 warnings

### Files with Warnings (Top 10)

- `src\intersect\suggestion.rs`: 2 warnings
- `src\index\builder.rs`: 2 warnings
- `src\serialize\chunked.rs`: 1 warnings
- `src\resolver\mod.rs`: 1 warnings

## Detailed Warning Categorization

### warning: method `next` can be confused for the standard trait method `std::iter::Iterator::next`

**Total Occurrences**: 6  
**Unique Files**: 4

#### `src\index\builder.rs`: 2 occurrences

- Line 129: this function has too many arguments (11/7)
- Line 171: this function has too many arguments (10/7)

#### `src\intersect\suggestion.rs`: 2 occurrences

- Line 167: the loop variable `i` is used to index `matrix`
- Line 171: the loop variable `j` is used to index `matrix`

#### `src\serialize\chunked.rs`: 1 occurrences

- Line 221: method `next` can be confused for the standard trait method `std::iter::Iterator::next`

#### `src\resolver\mod.rs`: 1 occurrences

- Line 17: module has the same name as its containing module

