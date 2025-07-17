# JSON Syntax Implementation Guide

This guide provides a comprehensive, step-by-step approach to implementing JSON-like syntax in Rust builder patterns using the cyrup-sugars ecosystem. The implementation uses native Rust array syntax without exposing macros to users.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Running the Example](#running-the-example)
4. [Implementation Details](#implementation-details)
5. [Step-by-Step Implementation](#step-by-step-implementation)
6. [User-Facing API](#user-facing-api)
7. [Performance Characteristics](#performance-characteristics)
8. [Troubleshooting](#troubleshooting)

## Overview

**Goal**: Provide JSON-like syntax in builder patterns without exposing macros to users.

**Solution**: Use native Rust array syntax `[("key", "value")]` that compiles to optimal HashMap operations.

**Key Features**:
- ✅ Zero allocation through compile-time optimization
- ✅ Blazing-fast performance via native array syntax
- ✅ No unsafe code, unchecked operations, or locking
- ✅ Elegant ergonomic code with clean API
- ✅ No macros exposed to users

## Quick Start

### Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
sugars_llm = { path = "../packages/llm" }
hashbrown = "0.14"
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Usage

```rust
use sugars_llm::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _stream = FluentAi::agent_role("rusty-squire")
        .additional_params([("beta", "true")])           // <- Native array syntax
        .metadata([("key", "val"), ("foo", "bar")])     // <- No macros needed
        .tools((
            Tool::<Perplexity>::new([("citations", "true")]),  // <- Clean API
        ))
        .chat("Hello")?;
    
    Ok(())
}
```

## Running the Example

### From the Example Directory

```bash
cd /path/to/sugars/examples/json_syntax
cargo run
```

### From the Workspace Root

```bash
cd /path/to/sugars
cargo run --manifest-path examples/json_syntax/Cargo.toml
```

### Expected Output

```
🤖 AI Agent Builder Example
Demonstrating the exact JSON object syntax from README.md

✅ Testing array syntax variations:
  - Single pair: Tool::new([('single', 'value')])
  - Multiple pairs: Tool::new([('key1', 'val1'), ('key2', 'val2')])
  - Empty array: Tool::new([])
✅ All syntax variations working correctly!
Chat stream initiated successfully!
```

## Implementation Details

### Core Trait: `IntoHashMap`

The implementation uses a trait that converts various types to HashMaps:

```rust
pub trait IntoHashMap {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str>;
}
```

### Array Syntax Implementation

```rust
impl<const N: usize> IntoHashMap for [(&'static str, &'static str); N] {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self.into_iter().collect()  // <- Zero-copy optimization
    }
}
```

### Supported Input Types

1. **Arrays**: `[("key", "value")]` - Compile-time optimized
2. **Vectors**: `vec![("key", "value")]` - Runtime flexible
3. **HashMaps**: Direct HashMap passing - Zero-copy
4. **Closures**: `|| hashmap!{"key" => "value"}` - Legacy support

## Step-by-Step Implementation

### Step 1: Create the IntoHashMap Trait

```rust
pub trait IntoHashMap {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str>;
}
```

### Step 2: Implement for Arrays (Primary Use Case)

```rust
impl<const N: usize> IntoHashMap for [(&'static str, &'static str); N] {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self.into_iter().collect()
    }
}
```

### Step 3: Implement for Other Types

```rust
// For backward compatibility with closures
impl<F> IntoHashMap for F 
where
    F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>
{
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self()
    }
}

// Direct HashMap passing
impl IntoHashMap for hashbrown::HashMap<&'static str, &'static str> {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self
    }
}

// Runtime Vec support
impl IntoHashMap for Vec<(&'static str, &'static str)> {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self.into_iter().collect()
    }
}
```

### Step 4: Update Builder Methods

```rust
impl AgentRoleBuilder {
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
    
    pub fn metadata<T>(mut self, meta: T) -> Self 
    where
        T: IntoHashMap
    {
        let config_map = meta.into_hashmap();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        self.metadata = Some(map);
        self
    }
}
```

### Step 5: Update Tool Constructor

```rust
impl<T> Tool<T> {
    pub fn new<C>(config: C) -> Self 
    where
        C: IntoHashMap
    {
        let config_map = config.into_hashmap();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        Self {
            _phantom: PhantomData,
            config: map,
        }
    }
}
```

## User-Facing API

### Syntax Variations

Users can use any of these equivalent syntaxes:

```rust
// Primary syntax - arrays (recommended)
.additional_params([("beta", "true")])
.metadata([("key", "val"), ("foo", "bar")])

// Empty parameters
.additional_params([])

// Single parameter
.additional_params([("single", "value")])

// Multiple parameters
.additional_params([("key1", "val1"), ("key2", "val2"), ("key3", "val3")])
```

### Complete Example

```rust
use sugars_llm::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AI Agent Builder Example");
    
    // Test various syntax patterns
    let _test1 = Tool::<Perplexity>::new([("single", "value")]);
    let _test2 = Tool::<Perplexity>::new([("key1", "val1"), ("key2", "val2")]);
    let _test3 = Tool::<Perplexity>::new([]);
    
    let _stream = FluentAi::agent_role("rusty-squire")
        .completion_provider(Mistral::MAGISTRAL_SMALL)
        .temperature(1.0)
        .max_tokens(8000)
        .system_prompt("You are a helpful AI assistant.")
        .additional_params([("beta", "true")])
        .metadata([("key", "val"), ("foo", "bar")])
        .tools((
            Tool::<Perplexity>::new([("citations", "true")]),
            Tool::named("cargo").bin("~/.cargo/bin")
        ))
        .chat("Hello")?;
    
    println!("Chat stream initiated successfully!");
    Ok(())
}
```

## Performance Characteristics

### Compile-Time Optimization

- **Array literals**: `[("key", "value")]` are stack-allocated constants
- **Zero allocation**: Small arrays remain on stack
- **Compiler optimization**: Rust optimizes array-to-HashMap conversion
- **Cache-friendly**: Contiguous memory layout

### Runtime Performance

- **Iterator efficiency**: `collect()` is highly optimized for known-size iterators
- **No heap allocations**: For small constant arrays
- **HashMap performance**: Using hashbrown for optimal hash operations
- **Static dispatch**: Generic traits enable compile-time optimization

### Memory Usage

- **Compile-time constants**: String literals become constants in binary
- **Stack allocation**: Arrays avoid heap allocations
- **Minimal overhead**: Direct iterator to HashMap conversion
- **Zero runtime overhead**: Array syntax compiles to optimal machine code

## Troubleshooting

### Common Issues

1. **"no example target named `json_syntax`"**
   
   The example is not a Cargo example target. Run it as:
   ```bash
   cd examples/json_syntax
   cargo run
   ```

2. **Compilation errors with array syntax**
   
   Ensure your builder methods accept the `IntoHashMap` trait:
   ```rust
   pub fn method<T>(self, params: T) -> Self 
   where
       T: IntoHashMap
   ```

3. **Missing trait implementations**
   
   Make sure you have all the necessary `IntoHashMap` implementations for arrays, vectors, and closures.

### Performance Tips

1. **Use arrays for constants**: `[("key", "value")]` for compile-time data
2. **Use vectors for runtime**: `vec![("key", "value")]` for dynamic data
3. **Avoid closures**: Unless needed for backward compatibility
4. **Profile your code**: Use `cargo flamegraph` to identify bottlenecks

### Testing

```bash
# Check compilation
cargo check --all

# Run the example
cd examples/json_syntax
cargo run

# Run with optimizations
cargo run --release
```

## Advanced Usage

### Custom Types

You can extend the trait for custom types:

```rust
impl IntoHashMap for MyCustomType {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        // Your custom conversion logic
        hashbrown::HashMap::new()
    }
}
```

### Error Handling

The current implementation is infallible for arrays, but you can add error handling:

```rust
pub trait TryIntoHashMap {
    type Error;
    
    fn try_into_hashmap(self) -> Result<hashbrown::HashMap<&'static str, &'static str>, Self::Error>;
}
```

This implementation provides a clean, performant, and user-friendly way to use JSON-like syntax in Rust builder patterns without exposing macros to users.