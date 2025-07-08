# [Cyrup Sugars](https://github.com/cyrup-ai/sugars)

[![Crates.io](https://img.shields.io/crates/v/cyrup_sugars.svg)](https://crates.io/crates/cyrup_sugars)
[![Documentation](https://docs.rs/cyrup_sugars/badge.svg)](https://docs.rs/cyrup_sugars)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

Syntactic sugar utilities for Rust - collections, async patterns, and macros.

## Features

This crate provides ergonomic utilities organized into feature-gated modules:

- **`collections`** - Enhanced collection types like `ZeroOneOrMany`, `OneOrMany`, and `ByteSize`
- **`async`** - Async utilities using`AsyncTask` and `AsyncStream`
- **`macros`** - Convenient macros for collections and async operations
- **`hashbrown-json`** - hashbrown HashMap macros with full JSON object support
- **`gix-interop`** - Git object ID optimized hash tables

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
cyrup_sugars = "0.1.0"
```

Or with specific features:

```toml
[dependencies]
# For amazing hashbrown + JSON support
cyrup_sugars = { version = "0.1", features = ["hashbrown-json"] }

# Or mix and match
cyrup_sugars = { version = "0.1", features = ["collections", "async"] }
```

## Examples

### Collections

```rust
use cyrup_sugars::collections::{ByteSizeExt, OneOrMany, ZeroOneOrMany};


// Flexible collections
let servers = ZeroOneOrMany();
servers.put_all(Provider::OpenAi, Provider::Ollama, Provider::Mistral);
```

### Async Operations

```rust
use cyrup_sugars::{AsyncTask, AsyncStream};

fn fetch_user_profile(user_id: u64) -> AsyncTask<UserProfile> {
    AsyncTask::from_api_call(format!("https://api.example.com/users/{}", user_id))
}

fn process_payments() -> AsyncTask<Vec<Payment>> {
    AsyncTask::from_database_query("SELECT * FROM payments WHERE status = 'pending'")
}

let user = fetch_user_profile(123)
    .on_success(|profile| println!("Welcome {}!", profile.name))
    .on_error(|e| println!("Error: {}", e))
    .await;

let payments = process_payments()
    .on_success(|payments| notify_accounting_team(payments.len()))
    .await;

let live_orders: AsyncStream<Order> = listen_for_orders();
live_orders
    .on_each(|order| process_order(order))
    .on_error(|e| alert_ops_team(e))
    .collect_async()
    .await;
    
    let error_result = error_task.await?;
    match error_result.into_inner() {
        Ok(val) => println!("Success: {}", val),
        Err(e) => println!("Error: {}", e),
    }
    
    Ok(())
}
```

## 🔥 Hashbrown JSON Object Syntax

The `hashbrown-json` feature enables clean **JSON object syntax** for all Rust types:

```rust
// Clean object literals
.additional_params({"beta" => "true"})
.metadata({"version" => "2.1.0", "env" => "prod"}) 
.config({"timeout" => 30, "retries" => 3})

// Everything serializes to perfect JSON automatically
// Works with: strings, numbers, booleans, enums, structs, traits, collections
```

The `NotResult` auto trait with negative impl ensures this at compile time:

```rust
pub auto trait NotResult {}
impl<T, E> !NotResult for Result<T, E> {}  // Negative impl
```

### Feature Gates

All modules are feature-gated for minimal dependencies:

```toml
# Minimal - only collections
cyrup_sugars = { version = "0.1", features = ["collections"] }

# Async support - choose your runtime:
cyrup_sugars = { version = "0.1", features = ["tokio-async"] }     # Tokio ecosystem
cyrup_sugars = { version = "0.1", features = ["std-async"] }       # Runtime-agnostic
cyrup_sugars = { version = "0.1", features = ["crossbeam-async"] } # Compute-heavy workloads

# 🔥 Hashbrown with JSON magic
cyrup_sugars = { version = "0.1", features = ["hashbrown-json"] }

# Everything
cyrup_sugars = { version = "0.1", features = ["full"] }
```

### Zero-Cost Abstractions

Collections like `ZeroOneOrMany` optimize for common cases:

- `None` variant uses zero heap allocations
- `One` variant stores the element directly
- `Many` variant pre-allocates capacity when transitioning from `One`

## Examples

Run the examples to see the library in action:

```bash
# Collections usage
cargo run --example collections_basic --features collections

# Async task pipeline
cargo run --example async_task_pipeline --features async

# Stream processing
cargo run --example async_stream_processing --features async

# Macro usage
cargo run --example macro_usage --features macros

# Full application
cargo run --example full_application --features full
```

## Testing

Run tests with full coverage:

```bash
# Run all tests
cargo test --all-features

# Generate coverage report
cargo tarpaulin --all-features --out Html
```

## Documentation

Generate and view documentation:

```bash
cargo doc --all-features --open
```

## Benchmarks

Run performance benchmarks:

```bash
cargo bench --all-features
```

## Contributing

Contributions are welcome! Please ensure:

1. All tests pass: `cargo test --all-features`
2. Code is formatted: `cargo fmt`
3. No clippy warnings: `cargo clippy --all-features -- -D warnings`
4. Documentation is updated: `cargo doc --all-features`
5. Examples work: `cargo run --example <name> --features <features>`

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
