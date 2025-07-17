# OneOrMany Usage Guide

This guide demonstrates how to use OneOrMany, a non-empty collection type that guarantees at least one element in the cyrup-sugars ecosystem.

## Table of Contents

1. [Overview](#overview)
2. [Creation Patterns](#creation-patterns)
3. [Iteration and Access](#iteration-and-access)
4. [Transformation Operations](#transformation-operations)
5. [JSON Object Syntax](#json-object-syntax)
6. [Merging and Combining](#merging-and-combining)
7. [Serialization Support](#serialization-support)
8. [Common Patterns](#common-patterns)
9. [Best Practices](#best-practices)
10. [Examples](#examples)

## Overview

OneOrMany is a non-empty collection that wraps `ZeroOneOrMany<T>`, ensuring the `None` variant is never used. It guarantees at least one element, with attempts to create an empty collection resulting in an `EmptyListError`.

### Key Features
- **Non-empty guarantee**: Always contains at least one element
- **Zero allocation**: Single items avoid heap allocation
- **Immutable operations**: All operations return new instances
- **Type safety**: Compile-time guarantee of non-emptiness
- **Serde support**: Serializes to JSON arrays with at least one element

## Creation Patterns

### Single Element

```rust
use sugars_collections::OneOrMany;

// Create with a single element
let single = OneOrMany::one("hello");
assert_eq!(single.len(), 1);
assert_eq!(single.first(), &"hello");
```

### Multiple Elements

```rust
use sugars_collections::OneOrMany;

// Create from a Vec (fails if empty)
let multiple = OneOrMany::many(vec![1, 2, 3]).unwrap();
assert_eq!(multiple.len(), 3);
assert_eq!(multiple.first(), &1);

// This would fail with EmptyListError
// let empty = OneOrMany::many(vec![]).unwrap(); // panics
```

### From Single Value

```rust
use sugars_collections::OneOrMany;

// Using From trait
let from_value: OneOrMany<i32> = 42.into();
assert_eq!(from_value.len(), 1);
```

### From Iterator

```rust
use sugars_collections::OneOrMany;

// From iterator (panics if empty)
let from_iter: OneOrMany<i32> = vec![1, 2, 3].into_iter().collect();
assert_eq!(from_iter.len(), 3);
```

## Iteration and Access

### Accessing Elements

```rust
use sugars_collections::OneOrMany;

let collection = OneOrMany::many(vec!["first", "second", "third"]).unwrap();

// Get first element (always exists)
let first = collection.first();
assert_eq!(first, &"first");

// Get remaining elements
let rest: Vec<&str> = collection.rest();
assert_eq!(rest, vec![&"second", &"third"]);

// Iterator over remaining elements
let rest_iter: Vec<&str> = collection.rest_iter().collect();
assert_eq!(rest_iter, vec![&"second", &"third"]);
```

### Iteration

```rust
use sugars_collections::OneOrMany;

let collection = OneOrMany::many(vec![1, 2, 3]).unwrap();

// Iterate by reference
for item in &collection {
    println!("Item: {}", item);
}

// Iterate by value (requires T: Clone + 'static)
for item in collection.clone() {
    println!("Owned item: {}", item);
}
```

## Transformation Operations

### Adding Elements

```rust
use sugars_collections::OneOrMany;

let original = OneOrMany::one(1);

// Add element to end
let with_pushed = original.with_pushed(2);
assert_eq!(with_pushed.len(), 2);

// Insert at specific position
let with_inserted = with_pushed.with_inserted(1, 10);
assert_eq!(with_inserted.len(), 3);
// Result: [1, 10, 2]
```

### Mapping Operations

```rust
use sugars_collections::OneOrMany;

let numbers = OneOrMany::many(vec![1, 2, 3]).unwrap();

// Map to new type
let strings = numbers.map(|n| n.to_string());
assert_eq!(strings.first(), &"1".to_string());

// Try map with error handling
let doubled: Result<OneOrMany<i32>, &str> = OneOrMany::many(vec![1, 2, 3])
    .unwrap()
    .try_map(|n| if n > 0 { Ok(n * 2) } else { Err("negative number") });
assert!(doubled.is_ok());
```

## JSON Object Syntax

### With Hashbrown HashMap

```rust
use sugars_collections::OneOrMany;
use hashbrown::HashMap;

let mut map = HashMap::new();
map.insert("key1", "value1");
map.insert("key2", "value2");

// Create from HashMap
let collection: OneOrMany<(&str, &str)> = OneOrMany::from_hashmap(map).unwrap();
assert_eq!(collection.len(), 2);
```

### With JSON Syntax (requires hashbrown-json feature)

```rust
use sugars_collections::OneOrMany;

// Using closure syntax for JSON objects
let collection: OneOrMany<(&str, &str)> = OneOrMany::from_json(|| {
    let mut map = hashbrown::HashMap::new();
    map.insert("beta", "true");
    map.insert("version", "2.1.0");
    map
}).unwrap();

assert_eq!(collection.len(), 2);
```

## Merging and Combining

### Merging Multiple Collections

```rust
use sugars_collections::OneOrMany;

let first = OneOrMany::one(1);
let second = OneOrMany::many(vec![2, 3]).unwrap();
let third = OneOrMany::one(4);

// Merge collections (requires T: Clone + 'static)
let merged = OneOrMany::merge(vec![first, second, third]).unwrap();
assert_eq!(merged.len(), 4);
// Result: [1, 2, 3, 4]
```

### Merging References

```rust
use sugars_collections::OneOrMany;

let first = OneOrMany::one(1);
let second = OneOrMany::many(vec![2, 3]).unwrap();

// Merge references (no Clone required)
let merged_refs = OneOrMany::merge_refs(vec![&first, &second]).unwrap();
assert_eq!(merged_refs.len(), 3);
```

## Serialization Support

### JSON Serialization

```rust
use sugars_collections::OneOrMany;
use serde_json;

let single = OneOrMany::one(42);
let json = serde_json::to_string(&single).unwrap();
assert_eq!(json, "[42]");

let multiple = OneOrMany::many(vec![1, 2, 3]).unwrap();
let json = serde_json::to_string(&multiple).unwrap();
assert_eq!(json, "[1,2,3]");
```

### JSON Deserialization

```rust
use sugars_collections::OneOrMany;
use serde_json;

// From single value
let single: OneOrMany<i32> = serde_json::from_str("[42]").unwrap();
assert_eq!(single.len(), 1);

// From array
let multiple: OneOrMany<i32> = serde_json::from_str("[1,2,3]").unwrap();
assert_eq!(multiple.len(), 3);

// Empty array fails
let empty_result: Result<OneOrMany<i32>, _> = serde_json::from_str("[]");
assert!(empty_result.is_err());
```

## Common Patterns

### Configuration Values

```rust
use sugars_collections::OneOrMany;

#[derive(Debug)]
struct Config {
    servers: OneOrMany<String>,
    ports: OneOrMany<u16>,
}

impl Config {
    fn new(servers: OneOrMany<String>, ports: OneOrMany<u16>) -> Self {
        Config { servers, ports }
    }
    
    fn primary_server(&self) -> &String {
        self.servers.first()
    }
    
    fn all_servers(&self) -> Vec<&String> {
        self.servers.iter().collect()
    }
}

// Usage
let config = Config::new(
    OneOrMany::many(vec!["server1.com".to_string(), "server2.com".to_string()]).unwrap(),
    OneOrMany::one(8080)
);

println!("Primary server: {}", config.primary_server());
```

### Builder Pattern Integration

```rust
use sugars_collections::OneOrMany;

struct HttpClient {
    endpoints: OneOrMany<String>,
}

impl HttpClient {
    fn new() -> HttpClientBuilder {
        HttpClientBuilder {
            endpoints: None,
        }
    }
}

struct HttpClientBuilder {
    endpoints: Option<OneOrMany<String>>,
}

impl HttpClientBuilder {
    fn endpoint(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.endpoints = Some(match self.endpoints {
            None => OneOrMany::one(url),
            Some(existing) => existing.with_pushed(url),
        });
        self
    }
    
    fn build(self) -> Result<HttpClient, &'static str> {
        let endpoints = self.endpoints.ok_or("At least one endpoint required")?;
        Ok(HttpClient { endpoints })
    }
}

// Usage
let client = HttpClient::new()
    .endpoint("https://api.example.com")
    .endpoint("https://api-backup.example.com")
    .build()
    .unwrap();
```

### Error Handling

```rust
use sugars_collections::OneOrMany;

enum ValidationError {
    EmptyField,
    InvalidFormat,
}

fn validate_inputs(inputs: Vec<String>) -> Result<OneOrMany<String>, ValidationError> {
    if inputs.is_empty() {
        return Err(ValidationError::EmptyField);
    }
    
    for input in &inputs {
        if input.is_empty() {
            return Err(ValidationError::InvalidFormat);
        }
    }
    
    OneOrMany::many(inputs).map_err(|_| ValidationError::EmptyField)
}

// Usage
let valid_inputs = vec!["input1".to_string(), "input2".to_string()];
let result = validate_inputs(valid_inputs).unwrap();
assert_eq!(result.len(), 2);
```

## Best Practices

### 1. Use OneOrMany for Required Fields

```rust
use sugars_collections::OneOrMany;

// Good: Express that at least one tag is required
struct BlogPost {
    title: String,
    content: String,
    tags: OneOrMany<String>, // At least one tag required
}

// Bad: Vec might be empty
struct BlogPostBad {
    title: String,
    content: String,
    tags: Vec<String>, // Could be empty
}
```

### 2. Handle Empty Collections Early

```rust
use sugars_collections::OneOrMany;

fn process_items(items: Vec<String>) -> Result<OneOrMany<String>, &'static str> {
    // Convert early to catch empty cases
    OneOrMany::many(items).map_err(|_| "No items provided")
}

// Usage
match process_items(vec![]) {
    Ok(items) => { /* Process items */ },
    Err(e) => println!("Error: {}", e),
}
```

### 3. Leverage Type Safety

```rust
use sugars_collections::OneOrMany;

// Function that requires at least one item
fn process_required_items(items: OneOrMany<String>) -> String {
    // No need to check if empty - guaranteed by type
    format!("Processing {} items, starting with: {}", items.len(), items.first())
}

// Compiler enforces non-empty requirement
let items = OneOrMany::many(vec!["item1".to_string(), "item2".to_string()]).unwrap();
let result = process_required_items(items);
```

### 4. Use Appropriate Conversion Methods

```rust
use sugars_collections::OneOrMany;

// Good: Use From for single items
let single: OneOrMany<i32> = 42.into();

// Good: Use many() for multiple items with error handling
let multiple = OneOrMany::many(vec![1, 2, 3]).unwrap();

// Good: Use try_from for Vec conversion
let from_vec: OneOrMany<i32> = vec![1, 2, 3].try_into().unwrap();
```

## Examples

### Load Balancer Configuration

```rust
use sugars_collections::OneOrMany;
use std::net::SocketAddr;

struct LoadBalancer {
    upstream_servers: OneOrMany<SocketAddr>,
    health_check_interval: u64,
}

impl LoadBalancer {
    fn new(servers: OneOrMany<SocketAddr>) -> Self {
        LoadBalancer {
            upstream_servers: servers,
            health_check_interval: 30,
        }
    }
    
    fn primary_server(&self) -> &SocketAddr {
        self.upstream_servers.first()
    }
    
    fn all_servers(&self) -> Vec<&SocketAddr> {
        self.upstream_servers.iter().collect()
    }
    
    fn add_server(self, server: SocketAddr) -> Self {
        LoadBalancer {
            upstream_servers: self.upstream_servers.with_pushed(server),
            ..self
        }
    }
}

// Usage
let primary = "127.0.0.1:8080".parse().unwrap();
let backup = "127.0.0.1:8081".parse().unwrap();

let lb = LoadBalancer::new(OneOrMany::one(primary))
    .add_server(backup);

println!("Primary: {}", lb.primary_server());
println!("All servers: {:?}", lb.all_servers());
```

### Plugin System

```rust
use sugars_collections::OneOrMany;

trait Plugin {
    fn name(&self) -> &str;
    fn process(&self, input: &str) -> String;
}

struct PluginManager {
    plugins: OneOrMany<Box<dyn Plugin>>,
}

impl PluginManager {
    fn new(plugin: Box<dyn Plugin>) -> Self {
        PluginManager {
            plugins: OneOrMany::one(plugin),
        }
    }
    
    fn add_plugin(self, plugin: Box<dyn Plugin>) -> Self {
        PluginManager {
            plugins: self.plugins.with_pushed(plugin),
        }
    }
    
    fn process(&self, input: &str) -> String {
        let mut result = input.to_string();
        for plugin in &self.plugins {
            result = plugin.process(&result);
        }
        result
    }
}

// Example plugin
struct UppercasePlugin;

impl Plugin for UppercasePlugin {
    fn name(&self) -> &str { "uppercase" }
    fn process(&self, input: &str) -> String { input.to_uppercase() }
}

// Usage
let manager = PluginManager::new(Box::new(UppercasePlugin));
let result = manager.process("hello world");
assert_eq!(result, "HELLO WORLD");
```

This guide demonstrates the flexibility and type safety of OneOrMany in handling collections that require at least one element, providing both compile-time guarantees and runtime efficiency.