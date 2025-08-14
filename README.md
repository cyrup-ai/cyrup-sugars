# [suga*rs*](https://github.com/cyrup-ai/sugars)

[![Crates.io](https://img.shields.io/crates/v/cyrup_sugars.svg)](https://crates.io/crates/cyrup_sugars)
[![Documentation](https://docs.rs/cyrup_sugars/badge.svg)](https://docs.rs/cyrup_sugars)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](https://opensource.org/licenses/MIT)

![Cyrup Sugars](./assets/suargs.jpg)

Syntactic sugar utilities for Rust - collections, async patterns, and macros.

## Features

- `collections` - Collection types: `ZeroOneOrMany`, `OneOrMany`, `ByteSize`
- `async` - Async utilities: `AsyncTask` and `AsyncStream`
- `macros` - Collection and async macros
- `array-tuples` - Array tuple syntax for collections
- `builders` - Builder traits: `MessageChunk`, `ChunkHandler`
- `gix-interop` - Git object hash tables

### Array Tuple Syntax

The `array-tuples` feature enables intuitive array tuple syntax in builder patterns:

```rust
let builder = FluentAi::agent_role("example")
    .additional_params([("beta", "true"), ("debug", "false")])
    .metadata([("key", "val"), ("foo", "bar")])
    .tools((
        Tool::<Perplexity>::new([("citations", "true"), ("format", "json")]),
    ));
```

**Implementation Pattern:**

```rust
/// Set additional parameters with array tuple syntax
pub fn additional_params<T>(mut self, params: T) -> Self
where
    T: IntoHashMap
{
    let config_map = params.into_hashmap();
    let mut map = HashMap::new();
    for (k, v) in config_map {
        map.insert(k.to_string(), Value::String(v.to_string()));
    }
    self.additional_params = Some(map);
    self
}
```

Usage: `.additional_params([("beta", "true"), ("debug", "false")])`

The array tuple syntax `[("key", "value")]` works seamlessly with the `IntoHashMap` trait, providing a clean and intuitive API for developers.

📖 **For complete implementation details, see the [Array Tuple Syntax Implementation Guide](./docs/ARRAY_TUPLE_SYNTAX.md)**

### AsyncTask Pattern

The `async` feature provides concrete async types that avoid boxed futures:

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;

// Single receiver
let task = AsyncTask::new(ZeroOneOrMany::one(rx));

// Multiple receivers (first result wins)
let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));

// From future
let task = AsyncTask::from_future(some_async_operation());

let result = task.await;
```

AsyncTask supports single and multiple receivers using the `ZeroOneOrMany` pattern, enabling race conditions, fallback patterns, and load balancing.

📖 **For complete usage examples, see the [AsyncTask Usage Guide](./docs/ASYNC_TASK.md)**

### Collection Types

The `collections` feature provides memory-efficient collection types for different scenarios:

```rust
use sugars_collections::{OneOrMany, ZeroOneOrMany};

// OneOrMany: Non-empty collections (guaranteed at least one element)
let servers = OneOrMany::many(vec!["server1", "server2"]).unwrap();
let primary = servers.first(); // Always exists

// ZeroOneOrMany: Flexible collections (zero allocations for None/One)
let middleware = ZeroOneOrMany::none();
let middleware = middleware.with_pushed("auth");
let middleware = middleware.with_pushed("cors");

match middleware {
    ZeroOneOrMany::None => println!("No middleware"),
    ZeroOneOrMany::One(mw) => println!("Single middleware: {}", mw),
    ZeroOneOrMany::Many(mws) => println!("Multiple middleware: {:?}", mws),
}
```

Both types support JSON serialization, builder patterns, and zero-allocation optimizations for small collections.

📖 **For complete guides, see [OneOrMany Guide](./docs/ONE_OR_MANY.md) and [ZeroOneOrMany Guide](./docs/ZERO_ONE_OR_MANY.md)**

### Builder Traits

The `builders` feature provides traits for message chunk handling:

- **`MessageChunk`** - Trait for types that can represent both success and error states
- **`ChunkHandler<T, E>`** - Handles streaming `Result<T, E>` by unwrapping to `T`

These traits enable consistent error handling patterns across builders with a single `on_chunk` method that handles both success and error cases.

📖 **For complete implementation guide, see the [ChunkHandler Pattern Guide](./docs/CHUNK_HANDLER.md)** - Step-by-step instructions for implementing the full pattern from scratch.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
cyrup_sugars = "0.1.0"
```

Or with specific features:

```toml
[dependencies]
cyrup_sugars = { version = "0.1", features = ["array-tuples"] }
```

## Example

```rust
let stream = FluentAi::agent_role("rusty-squire")
    .completion_provider(Mistral::MagistralSmall)
    .temperature(1.0)
    .max_tokens(8000)
    .system_prompt("Act as a Rust developers 'right hand man'.
        You possess deep expertise in using tools to research rust, cargo doc and github libraries.
        You are a patient and thoughtful software artisan; a master of sequential thinking and step-by-step reasoning.
        You excel in compilation triage ...

        ...
        ...

        Today is {{ date }}

        ~ Be Useful, Not Thorough")
    .context( // trait Context
        Context<File>::of("/home/kloudsamurai/ai_docs/mistral_agents.pdf"),
        Context<Files>::glob("/home/kloudsamurai/cyrup-ai/**/*.{md,txt}"),
        Context<Directory>::of("/home/kloudsamurai/cyrup-ai/agent-role/ambient-rust"),
        Context<Github>::glob("/home/kloudsamurai/cyrup-ai/**/*.{rs,md}")
    )
    .mcp_server<Stdio>::bin("/user/local/bin/sweetmcp").init("cargo run -- --stdio")
    .tools( // trait Tool
        Tool<Perplexity>::new([
            ("citations", "true"), ("format", "json")
        ]),
        Tool::named("cargo").bin("~/.cargo/bin").description("cargo --help".exec_to_text())
    ) // ZeroOneOrMany `Tool` || `McpTool` || NamedTool (WASM)

    .additional_params([("beta", "true"), ("debug", "false")])
    .memory(Library::named("obsidian_vault"))
    .metadata([("key", "val"), ("foo", "bar")])
    .on_tool_result(|results| {
        // do stuff
    })
    .on_conversation_turn(|conversation, agent| {
        log.info("Agent: " + conversation.last().message())
        agent.chat(process_turn()) // your custom logic
    })
    .on_chunk(|result| match result {  // Handle Result<T, E> unwrapping
        Ok(chunk) => {
            println!("{}", chunk);
            chunk
        },
        Err(e) => ConversationChunk::bad_chunk(e)  // Convert error to bad chunk
    })
    .into_agent() // Agent Now
    .conversation_history(MessageRole::User => "What time is it in Paris, France",
            MessageRole::System => "The USER is inquiring about the time in Paris, France. Based on their IP address, I see they are currently in Las Vegas, Nevada, USA. The current local time is 16:45",
            MessageRole::Assistant => "It’s 1:45 AM CEST on July 7, 2025, in Paris, France. That’s 9 hours ahead of your current time in Las Vegas.")
    .chat("Hello")? // AsyncStream<MessageChunk
    .collect();

## Working Examples

### Message Chunk Handling

The `builders` feature provides powerful traits for handling streaming message chunks:

```rust
use cyrup_sugars::prelude::*;
use sugars_llm::*;

// MessageChunk trait - implemented by chunk types
pub trait MessageChunk: Sized {
    fn bad_chunk(error: String) -> Self;  // Create error chunk
    fn error(&self) -> Option<&str>;      // Get error if present
    fn is_error(&self) -> bool;           // Check if error chunk
}

// ChunkHandler trait - processes Result<T, E> streams
let builder = builder
    .on_chunk(|result| match result {
        Ok(chunk) => chunk,
        Err(e) => ConversationChunk::bad_chunk(e)
    });
```

### Array Tuple Syntax

```rust
// All these patterns work with array tuple syntax:
Tool::<Perplexity>::new([("citations", "true")])
    .additional_params([("beta", "true"), ("debug", "false")])
    .metadata([("key", "val"), ("foo", "bar")])
```

### Run Examples

```bash
# Array tuple syntax with on_chunk method
cd examples/array_tuple_syntax && cargo run

# Async task pipeline
cd examples/async_task_example && cargo run

# Collection types
cd examples/one_or_many_example && cargo run
cd examples/zero_one_or_many_example && cargo run
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
