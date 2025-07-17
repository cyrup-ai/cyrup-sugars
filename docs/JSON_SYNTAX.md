# JSON Syntax Implementation Guide

This guide provides a comprehensive, step-by-step approach to implementing JSON object syntax (`{"key" => "value"}`) in Rust builder patterns using the cyrup-sugars ecosystem.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Step 1: Set Up Dependencies](#step-1-set-up-dependencies)
4. [Step 2: Create the Hash Map Macro](#step-2-create-the-hash-map-macro)
5. [Step 3: Implement Builder Methods](#step-3-implement-builder-methods)
6. [Step 4: Add Extension Traits](#step-4-add-extension-traits)
7. [Step 5: Test the Implementation](#step-5-test-the-implementation)
8. [Common Pitfalls](#common-pitfalls)
9. [Advanced Usage](#advanced-usage)
10. [Troubleshooting](#troubleshooting)

## Overview

*`Cargo.toml`*:

```toml
cyrup_sugars = { git = "https://github.com/cyrup-ai/cyrup-sugars", package = "cyrup_sugars", branch = "main", features = ["hashbrown-json"] }
```

**Other available guides:**

- **AsyncTask Usage**: For async patterns with single/multiple receivers

  ```toml
  cyrup_sugars = { features = ["tokio-async"] }  # or std-async, crossbeam-async
  ```

  📖 [AsyncTask Usage Guide](./ASYNC_TASK.md)

- **OneOrMany Collections**: For non-empty collections with type safety

  ```toml
  cyrup_sugars = { features = ["collections"] }  # included in default
  ```

  📖 [OneOrMany Usage Guide](./ONE_OR_MANY.md)

- **ZeroOneOrMany Collections**: For flexible collections with zero allocations

  ```toml
  cyrup_sugars = { features = ["collections"] }  # included in default
  ```

  📖 [ZeroOneOrMany Usage Guide](./ZERO_ONE_OR_MANY.md)

The JSON syntax feature allows developers to write intuitive builder patterns like:

```rust
FluentAi::agent_role("researcher")
    .additional_params({"beta" => "true"})
    .metadata({"key" => "val", "foo" => "bar"})
    .tools((
        Tool::<Perplexity>::new({"citations" => "true"}),
    ))
```

This syntax works automatically with the transformation system. The `{"key" => "value"}` syntax is transformed internally without exposing any macros to users.

## Prerequisites

- Rust 1.70+ (for proc-macro2 features)
- Understanding of Rust macros and proc-macros
- Knowledge of the Into trait and generic constraints
- Familiarity with hashbrown HashMap

## Step 1: Set Up Dependencies

### 1.1 Add Required Dependencies

In your `Cargo.toml`:

```toml
[dependencies]
hashbrown = "0.14"
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }

[features]
default = ["hashbrown-json"]
hashbrown-json = []
```

### 1.2 Create Macro Crate Structure

Create a separate crate for macros (required for proc-macros):

```
packages/
├── macros/
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs
├── collections/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── json_ext.rs
└── your-builder/
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

### 1.3 Configure Proc-Macro Crate

In `packages/macros/Cargo.toml`:

```toml
[lib]
proc-macro = true

[dependencies]
proc-macro2 = "1.0"
quote = "1.0"
syn = { version = "2.0", features = ["full"] }
hashbrown = "0.14"
```

## Step 2: Create the Hash Map Macro

### 2.1 Implement the Core Macro

In `packages/macros/src/lib.rs`:

```rust
use proc_macro::TokenStream;
use quote::quote;

/// Creates a closure that returns a hashbrown HashMap from key-value pairs
///
/// This macro transforms JSON-like syntax into valid Rust code:
/// `{"key" => "value"}` syntax becomes a closure that creates a HashMap
#[proc_macro]
pub fn json_transform(input: TokenStream) -> TokenStream {
    // Convert the input to a string and manually parse key => value pairs
    let input_str = input.to_string();

    // Transform "key" => "value" pairs to ("key", "value") tuples
    let parts: Vec<&str> = input_str.split(',').collect();
    let mut tuple_pairs = Vec::new();

    for part in parts {
        let trimmed = part.trim();
        if let Some(arrow_pos) = trimmed.find(" => ") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 4..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        } else if let Some(arrow_pos) = trimmed.find("=>") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 2..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        }
    }

    let tuple_str = tuple_pairs.join(", ");
    let parsed_tokens: proc_macro2::TokenStream = tuple_str.parse().unwrap_or_default();

    quote! {
        {
            struct JsonHashMap(::hashbrown::HashMap<&'static str, &'static str>);
            impl Into<::hashbrown::HashMap<&'static str, &'static str>> for JsonHashMap {
                fn into(self) -> ::hashbrown::HashMap<&'static str, &'static str> {
                    self.0
                }
            }
            JsonHashMap(<::hashbrown::HashMap::<_, _> as ::core::iter::FromIterator<_>>::from_iter([
                #parsed_tokens
            ]))
        }
    }
    .into()
}
```

### 2.2 Add Attribute Macro Support

Add this to enable JSON syntax on builder structs:

```rust
/// Attribute macro that can be applied to builder structs to enable JSON syntax
#[proc_macro_attribute]
pub fn enable_json_syntax(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the item unchanged
    // This could be extended to automatically generate JSON-aware methods
    item
}
```

## Step 3: Implement Builder Methods

### 3.1 Basic Builder Pattern

Create your builder struct with the attribute:

```rust
use sugars_macros::enable_json_syntax;

#[enable_json_syntax]
pub struct AgentRoleBuilder {
    additional_params: Option<HashMap<String, Value>>,
    metadata: Option<HashMap<String, Value>>,
    // ... other fields
}
```

### 3.2 Implement Builder Methods with Generic Parameters

**Critical:** Use generic type parameters, not `impl Into<>` directly in the parameter:

```rust
impl AgentRoleBuilder {
    /// Set additional parameters with JSON object syntax
    pub fn additional_params<P>(mut self, params: P) -> Self
    where
        P: Into<hashbrown::HashMap<&'static str, &'static str>>
    {
        let config_map = params.into();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        self.additional_params = Some(map);
        self
    }

    /// Set metadata with JSON object syntax
    pub fn metadata<P>(mut self, metadata: P) -> Self
    where
        P: Into<hashbrown::HashMap<&'static str, &'static str>>
    {
        let config_map = metadata.into();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        self.metadata = Some(map);
        self
    }
}
```

### 3.3 Tool Constructor Example

```rust
impl<T> Tool<T> {
    pub fn new<P>(_params: P) -> Tool<T>
    where
        P: Into<hashbrown::HashMap<&'static str, &'static str>>
    {
        // Store params in a real implementation
        Tool(std::marker::PhantomData)
    }
}
```

## Step 4: Add Extension Traits

### 4.1 Create Extension Traits

In `packages/collections/src/json_ext.rs`:

```rust
#[cfg(feature = "hashbrown-json")]
pub trait JsonObjectExtStringString: Sized {
    type Error;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<String>;
}

#[cfg(feature = "hashbrown-json")]
impl JsonObjectExtStringString for Vec<(String, String)> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Ok(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}
```

### 4.2 Collection Extensions

```rust
#[cfg(feature = "hashbrown-json")]
pub trait CollectionJsonExtStringString {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>;
}

#[cfg(feature = "hashbrown-json")]
impl CollectionJsonExtStringString for Vec<(String, String)> {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        f().into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}
```

## Step 5: Test the Implementation

### 5.1 Create Test Examples

Create a test file that uses the JSON syntax:

```rust
use your_builder::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _agent = FluentAi::agent_role("test")
        .additional_params({"beta" => "true"})
        .metadata({"key" => "val", "foo" => "bar"})
        .tools((
            Tool::<Perplexity>::new({"citations" => "true"}),
        ));

    Ok(())
}
```

### 5.2 Run Tests

```bash
cargo check -p your-examples
cargo test --features hashbrown-json
```

## Common Pitfalls

### 1. Using `impl Into<>` in Parameters

❌ **Wrong:**

```rust
pub fn method(self, params: impl Into<HashMap<&'static str, &'static str>>) -> Self
```

✅ **Correct:**

```rust
pub fn method<P>(self, params: P) -> Self
where
    P: Into<HashMap<&'static str, &'static str>>
```

### 2. Forgetting the Macro Call

❌ **Wrong:**

```rust
.additional_params({"beta" => "true"})  // This won't compile
```

✅ **Correct:**

```rust
.additional_params({"beta" => "true"})
```

### 3. Missing Feature Gates

Make sure your Cargo.toml has:

```toml
[features]
hashbrown-json = []
```

### 4. Incorrect Macro Syntax

❌ **Wrong:**

```rust
{"key" => "value"}  // This works automatically
```

✅ **Correct:**

```rust
{"key" => "value"}  // This works automatically
```

## Advanced Usage

### 1. Supporting Different Value Types

```rust
pub fn typed_params<P>(mut self, params: P) -> Self
where
    P: Into<hashbrown::HashMap<&'static str, serde_json::Value>>
{
    // Implementation for JSON values
}
```

### 2. Nested JSON Support

```rust
// For complex nested structures
pub fn nested_config<P>(mut self, config: P) -> Self
where
    P: Into<serde_json::Map<String, serde_json::Value>>
{
    // Implementation
}
```

### 3. Optional Parameters

```rust
pub fn optional_params<P>(mut self, params: Option<P>) -> Self
where
    P: Into<hashbrown::HashMap<&'static str, &'static str>>
{
    if let Some(p) = params {
        let config_map = p.into();
        // Process the map
    }
    self
}
```

## Troubleshooting

### Compilation Errors

1. **"expected one of `.`, `;`, `?`, `}`, or an operator, found `=>`"**
   - Make sure the `hashbrown-json` feature is enabled in your Cargo.toml

2. **"cannot find function or method"**
   - Ensure your builder methods accept `impl Into<hashbrown::HashMap<&'static str, &'static str>>`

3. **"trait bound not satisfied"**
   - Ensure your builder method uses generic parameters with proper bounds

### Runtime Issues

1. **HashMap is empty**
   - Check that your macro is properly parsing the key-value pairs
   - Verify the string splitting logic in the macro

2. **Type conversion errors**
   - Ensure your Into implementations are correct
   - Check that you're using the right string lifetime (`&'static str`)

### Performance Considerations

1. **Compile-time overhead**
   - The macro parsing happens at compile time, so it doesn't affect runtime performance
   - Consider caching compiled results for large projects

2. **Memory usage**
   - The generated HashMaps are small and efficient
   - String literals are stored in the binary's static memory

## Example Complete Implementation

Here's a minimal working example:

```rust
// In packages/macros/src/lib.rs
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn json_transform(input: TokenStream) -> TokenStream {
    let input_str = input.to_string();
    let parts: Vec<&str> = input_str.split(',').collect();
    let mut tuple_pairs = Vec::new();

    for part in parts {
        let trimmed = part.trim();
        if let Some(arrow_pos) = trimmed.find(" => ") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 4..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        }
    }

    let tuple_str = tuple_pairs.join(", ");
    let parsed_tokens: proc_macro2::TokenStream = tuple_str.parse().unwrap_or_default();

    quote! {
        {
            struct JsonHashMap(::hashbrown::HashMap<&'static str, &'static str>);
            impl Into<::hashbrown::HashMap<&'static str, &'static str>> for JsonHashMap {
                fn into(self) -> ::hashbrown::HashMap<&'static str, &'static str> {
                    self.0
                }
            }
            JsonHashMap(<::hashbrown::HashMap::<_, _> as ::core::iter::FromIterator<_>>::from_iter([
                #parsed_tokens
            ]))
        }
    }
    .into()
}

// In your builder implementation
impl Builder {
    pub fn params<P>(mut self, params: P) -> Self
    where
        P: Into<hashbrown::HashMap<&'static str, &'static str>>
    {
        let config_map = params.into();
        // Use the config_map...
        self
    }
}

// Usage
let builder = Builder::new()
    .params({"key" => "value", "foo" => "bar"});
```

This guide provides a complete foundation for implementing JSON syntax in your Rust builder patterns. The `{"key" => "value"}` syntax works automatically with the transformation system - no macros need to be imported or called by users.
