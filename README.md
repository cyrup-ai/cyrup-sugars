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
- `hashbrown-json` - JSON object syntax for collections
- `gix-interop` - Git object hash tables

## Quick Import with Prelude

Get started quickly by importing everything you need:

```rust
use cyrup_sugars::prelude::*;

// Now you have access to:
// - Collection types: OneOrMany, ZeroOneOrMany, ByteSize
// - Async utilities: AsyncTask, AsyncStream, AsyncResult
// - Closure macros: on_result!, on_chunk!, on_error!, await_result!, await_ok!
// - JSON syntax macros: hash_map! (with hashbrown-json feature)
```

### JSON Object Syntax

The `hashbrown-json` feature enables intuitive JSON-like syntax in builder patterns:

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
        Tool<Perplexity>::new({
            "citations" => "true"
        }),
        Tool::named("cargo").bin("~/.cargo/bin").description("cargo --help".exec_to_text())
    ) // ZeroOneOrMany `Tool` || `McpTool` || NamedTool (WASM)

    .additional_params({"beta" =>  "true"})
    .memory(Library::named("obsidian_vault"))
    .metadata({ "key" => "val", "foo" => "bar" })
    .on_tool_result(|results| {
        // do stuff
    })
    .on_conversation_turn(|conversation, agent| {
        log.info("Agent: " + conversation.last().message())
        agent.chat(process_turn()) // your custom logic
    })
    .on_chunk(|chunk| {          // unwrap chunk closure :: NOTE: THIS MUST PRECEDE .chat()
        Ok => chunk.into()       // `.chat()` returns AsyncStream<MessageChunk> vs. AsyncStream<Result<MessageChunk>>
        println!("{}", chunk);   // stream response here or from the AsyncStream .chat() returns
    })
    .into_agent() // Agent Now
    .conversation_history(MessageRole::User => "What time is it in Paris, France",
            MessageRole::System => "The USER is inquiring about the time in Paris, France. Based on their IP address, I see they are currently in Las Vegas, Nevada, USA. The current local time is 16:45",
            MessageRole::Assistant => "It’s 1:45 AM CEST on July 7, 2025, in Paris, France. That’s 9 hours ahead of your current time in Las Vegas.")
    .chat(|conversation| {
        let user_input = conversation.latest_user_message();
        
        if user_input.contains("finished") {
            ChatLoop::Break
        } else {
            ChatLoop::Reprompt("continue. use sequential thinking")
        }
    })?

// Full Example with Pure ChatLoop Pattern:
FluentAi::agent_role("helpful assistant")
    .completion_provider(Providers::OpenAI)
    .model(Models::Gpt4OMini)
    .temperature(0.7)
    .on_chunk(|chunk| {
        // Real-time streaming - print each token as it arrives
        // All formatting and coloring happens automatically here
        print!("{}", chunk);
        io::stdout().flush().unwrap();
    })
    .chat(|conversation| {
        let user_input = conversation.latest_user_message();
        
        // Pure logic - no formatting, just conversation flow control
        match user_input.to_lowercase().as_str() {
            "quit" | "exit" | "bye" => {
                ChatLoop::Break
            },
            input if input.starts_with("/help") => {
                ChatLoop::Reprompt("Available commands: /help, quit/exit/bye, or just chat normally!".to_string())
            },
            input if input.contains("code") => {
                let response = format!(
                    "I see you mentioned code! Here's a Rust example: fn main() {{ println!(\"Hello!\"); }} Need help with a specific language?"
                );
                ChatLoop::Reprompt(response)
            },
            _ => {
                // Simple response - builder handles all formatting automatically
                let response = format!(
                    "I understand: '{}'. How can I help you further?", 
                    user_input
                );
                ChatLoop::Reprompt(response)
            }
        }
    })?
    .collect();
```

The `{"key" => "value"}` syntax works seamlessly with the transformation system, providing a clean and intuitive API for developers.

📖 **For complete implementation details, see the [JSON Syntax Implementation Guide](./docs/JSON_SYNTAX.md)**

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

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
cyrup_sugars = "0.1.3"
```

Or with specific features:

```toml
[dependencies]
cyrup_sugars = { version = "0.1.3", features = ["hashbrown-json"] }
```

## Example

```rust
use cyrup_sugars::prelude::*;

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
        Tool<Perplexity>::new({
            "citations" => "true"
        }),
        Tool::named("cargo").bin("~/.cargo/bin").description("cargo --help".exec_to_text())
    ) // ZeroOneOrMany `Tool` || `McpTool` || NamedTool (WASM)

    .additional_params({"beta" =>  "true"})
    .memory(Library::named("obsidian_vault"))
    .metadata({ "key" => "val", "foo" => "bar" })
    .on_tool_result(|results| {
        // do stuff
    })
    .on_conversation_turn(|conversation, agent| {
        log.info("Agent: " + conversation.last().message())
        agent.chat(process_turn()) // your custom logic
    })
    .on_chunk(|chunk| {          // unwrap chunk closure :: NOTE: THIS MUST PRECEDE .chat()
        Ok => chunk.into()       // `.chat()` returns AsyncStream<MessageChunk> vs. AsyncStream<Result<MessageChunk>>
        println!("{}", chunk);   // stream response here or from the AsyncStream .chat() returns
    })
    .into_agent() // Agent Now
    .conversation_history(MessageRole::User => "What time is it in Paris, France",
            MessageRole::System => "The USER is inquiring about the time in Paris, France. Based on their IP address, I see they are currently in Las Vegas, Nevada, USA. The current local time is 16:45",
            MessageRole::Assistant => "It’s 1:45 AM CEST on July 7, 2025, in Paris, France. That’s 9 hours ahead of your current time in Las Vegas.")
    .chat("Hello")? // AsyncStream<MessageChunk
    .collect();

Run the examples to see the library in action:

```bash
# JSON object syntax
cd examples/json_syntax && cargo run

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
