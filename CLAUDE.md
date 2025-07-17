# CLAUDE.md - Development Guidelines

## Testing with Nextest

This project uses cargo-nextest for comprehensive feature testing. All features must be tested in isolation and in combination.

### Install Nextest
```bash
cargo install cargo-nextest --locked
```

### Feature Testing Commands

Run these commands to test all feature combinations:

```bash
# No features (minimal)
cargo nextest run --no-default-features

# Only std
cargo nextest run --no-default-features --features std

# Collections only  
cargo nextest run --no-default-features --features collections

# Async only
cargo nextest run --no-default-features --features async

# Macros only
cargo nextest run --no-default-features --features macros

# Default features
cargo nextest run

# Tokio async backend
cargo nextest run --no-default-features --features tokio-async

# Std async backend
cargo nextest run --no-default-features --features std-async

# Crossbeam async backend
cargo nextest run --no-default-features --features crossbeam-async

# Serde support
cargo nextest run --no-default-features --features collections,serde

# Hashbrown JSON support
cargo nextest run --no-default-features --features hashbrown-json

# Gix interop
cargo nextest run --no-default-features --features gix-interop

# Collections + Serde
cargo nextest run --no-default-features --features collections,serde

# Collections + Hashbrown JSON
cargo nextest run --no-default-features --features collections,hashbrown-json

# Async + Collections
cargo nextest run --no-default-features --features async,collections

# Macros + Collections
cargo nextest run --no-default-features --features macros,collections

# All async backends
cargo nextest run --no-default-features --features tokio-async,std-async,crossbeam-async

# Full feature set
cargo nextest run --features full

# Complex combinations
cargo nextest run --no-default-features --features async,collections,macros
cargo nextest run --no-default-features --features gix-interop,collections,macros
cargo nextest run --no-default-features --features hashbrown-json,collections,macros
```

### Compilation Checks
```bash
# Examples feature compilation check
cargo check --features examples
```

### Required Test Coverage

Every feature must have:
- ✅ Isolation tests (feature works alone)
- ✅ Integration tests (feature works with others)
- ✅ Regression tests (removing feature doesn't break others)
- ✅ Edge case tests (empty collections, error conditions)
- ✅ Serde compatibility tests (when applicable)

### Feature Matrix

- `std` - Base standard library support
- `collections` - ZeroOneOrMany, OneOrMany, ByteSize
- `async` - AsyncTask, AsyncResult, AsyncStream  
- `macros` - Collection and closure macros
- `tokio-async` - Tokio async backend
- `std-async` - Standard async backend
- `crossbeam-async` - Crossbeam async backend
- `serde` - Serialization support
- `hashbrown-json` - Hashbrown HashMap with JSON
- `gix-interop` - Git object hash tables
- `full` - All features combined
- `examples` - Example code compilation

### Test Files

- `tests/collections_features.rs` - Collections feature tests
- `tests/async_features.rs` - Async feature tests  
- `tests/macros_features.rs` - Macros feature tests
- `tests/gix_interop_features.rs` - Gix interop tests
- `tests/feature_combinations.rs` - Integration tests

### Nextest Configuration

Located in `.config/nextest.toml` with profiles for each feature set.

## Code Quality Standards

- ✅ 0 errors, 0 warnings on `cargo check`
- ✅ All features work in isolation
- ✅ All feature combinations work
- ✅ Production-ready code quality
- ✅ Comprehensive test coverage