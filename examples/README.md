# Examples

This directory contains example implementations demonstrating the cyrup-sugars library features.

## Structure

- `array_tuple_syntax/` - Demonstrates array tuple syntax in builder patterns
- `async_task_example/` - Shows AsyncTask usage with single and multiple receivers
- `one_or_many_example/` - Shows OneOrMany usage for non-empty collections
- `zero_one_or_many_example/` - Shows ZeroOneOrMany usage for flexible collections

## Running Examples

### Array Tuple Syntax Example

```bash
cd examples/array_tuple_syntax
cargo run
```

This demonstrates the `[("key", "value")]` syntax working seamlessly with builder patterns.

### AsyncTask Example

```bash
cd examples/async_task_example
cargo run
```

This shows:
- Single receiver usage
- Multiple receivers (race condition)
- From future pattern
- From value pattern  
- Parallel processing
- Timeout patterns

### OneOrMany Example

```bash
cd examples/one_or_many_example
cargo run
```

This demonstrates:
- Non-empty collection guarantees
- Single and multiple element handling
- Transformation operations
- Builder pattern integration
- Error handling for empty collections

### ZeroOneOrMany Example

```bash
cd examples/zero_one_or_many_example
cargo run
```

This shows:
- All three variants (None, One, Many)
- Pattern matching for proper handling
- Memory-efficient collection handling
- JSON serialization for all variants
- Event handling systems

## Features Demonstrated

### Array Tuple Syntax
- Builder patterns with intuitive array tuple syntax
- Standard Rust syntax without transformations
- Clean API for developers

### AsyncTask Patterns
- Single receiver handling
- Multiple receiver race conditions
- Channel-based async communication
- Concrete types without boxed futures
- ZeroOneOrMany pattern usage

### Collection Types
- OneOrMany: Non-empty collections with type safety
- ZeroOneOrMany: Flexible collections with zero allocations
- Memory-efficient storage for different collection sizes
- Pattern matching for proper variant handling
- JSON serialization support