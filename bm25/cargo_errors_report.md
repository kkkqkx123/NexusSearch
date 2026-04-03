# Cargo Check Error Analysis Report

## Summary

- **Total Errors**: 3
- **Total Warnings**: 0
- **Total Issues**: 3
- **Unique Error Patterns**: 2
- **Unique Warning Patterns**: 0
- **Files with Issues**: 2

## Error Statistics

**Total Errors**: 3

### Error Type Breakdown

- **error[E0308]**: 2 errors
- **error[E0596]**: 1 errors

### Files with Errors (Top 10)

- `src\index\manager.rs`: 2 errors
- `src\service\grpc.rs`: 1 errors

## Warning Statistics

**Total Warnings**: 0

## Detailed Error Categorization

### error[E0308]: arguments to this function are incorrect

**Total Occurrences**: 2  
**Unique Files**: 1

#### `src\index\manager.rs`: 2 occurrences

- Line 196: arguments to this function are incorrect
- Line 212: arguments to this function are incorrect

### error[E0596]: cannot borrow `writer` as mutable, as it is not declared as mutable: cannot borrow as mutable

**Total Occurrences**: 1  
**Unique Files**: 1

#### `src\service\grpc.rs`: 1 occurrences

- Line 228: cannot borrow `writer` as mutable, as it is not declared as mutable: cannot borrow as mutable

