# ZeroOneOrMany Usage Guide

This guide demonstrates how to use ZeroOneOrMany, a flexible collection type that can hold zero, one, or many values with optimal memory usage in the cyrup-sugars ecosystem.

## Table of Contents

1. [Overview](#overview)
2. [Creation Patterns](#creation-patterns)
3. [Variant Handling](#variant-handling)
4. [Iteration and Access](#iteration-and-access)
5. [Transformation Operations](#transformation-operations)
6. [JSON Object Syntax](#json-object-syntax)
7. [Merging and Combining](#merging-and-combining)
8. [Serialization Support](#serialization-support)
9. [Common Patterns](#common-patterns)
10. [Best Practices](#best-practices)
11. [Examples](#examples)

## Overview

ZeroOneOrMany is a collection that can hold zero, one, or many values of type `T`. It provides an efficient way to represent collections that might be empty, contain a single element, or contain multiple elements, minimizing heap allocations.

### Variants
- **None**: Empty collection with zero elements (zero heap allocations)
- **One(T)**: Collection with exactly one element (zero heap allocations)
- **Many(Vec<T>)**: Collection with multiple elements (single heap allocation)

### Key Features
- **Memory efficient**: Zero allocations for None and One variants
- **Immutable operations**: All operations return new instances
- **Flexible**: Handles all collection size scenarios
- **Type safe**: Pattern matching enforces proper handling
- **Serde support**: Serializes to JSON arrays, null, or single values

## Creation Patterns

### Empty Collection

```rust
use sugars_collections::ZeroOneOrMany;

// Create empty collection
let empty = ZeroOneOrMany::none();
assert_eq!(empty.len(), 0);
assert!(empty.is_empty());
```

### Single Element

```rust
use sugars_collections::ZeroOneOrMany;

// Create with single element
let single = ZeroOneOrMany::one("hello");
assert_eq!(single.len(), 1);
assert_eq!(single.first(), Some(&"hello"));
```

### Multiple Elements

```rust
use sugars_collections::ZeroOneOrMany;

// Create from Vec
let multiple = ZeroOneOrMany::many(vec![1, 2, 3]);
assert_eq!(multiple.len(), 3);
assert_eq!(multiple.first(), Some(&1));

// Empty Vec becomes None
let empty_vec = ZeroOneOrMany::many(vec![]);
assert!(empty_vec.is_empty());
```

### From Iterator

```rust
use sugars_collections::ZeroOneOrMany;

// From iterator
let from_iter: ZeroOneOrMany<i32> = vec![1, 2, 3].into_iter().collect();
assert_eq!(from_iter.len(), 3);

// Empty iterator becomes None
let empty_iter: ZeroOneOrMany<i32> = vec![].into_iter().collect();
assert!(empty_iter.is_empty());
```

## Variant Handling

### Pattern Matching

```rust
use sugars_collections::ZeroOneOrMany;

let collection = ZeroOneOrMany::many(vec![1, 2, 3]);

match collection {
    ZeroOneOrMany::None => println!("Empty collection"),
    ZeroOneOrMany::One(item) => println!("Single item: {}", item),
    ZeroOneOrMany::Many(items) => println!("Multiple items: {:?}", items),
}
```

### Safe Access Methods

```rust
use sugars_collections::ZeroOneOrMany;

let collection = ZeroOneOrMany::one(42);

// Safe access to first element
if let Some(first) = collection.first() {
    println!("First element: {}", first);
}

// Check if empty
if collection.is_empty() {
    println!("Collection is empty");
} else {
    println!("Collection has {} elements", collection.len());
}
```

## Iteration and Access

### Accessing Elements

```rust
use sugars_collections::ZeroOneOrMany;

let collection = ZeroOneOrMany::many(vec!["first", "second", "third"]);

// Get first element (Option)
let first = collection.first();
assert_eq!(first, Some(&"first"));

// Get remaining elements
let rest: Vec<&str> = collection.rest();
assert_eq!(rest, vec![&"second", &"third"]);

// Iterator over remaining elements
let rest_iter: Vec<&str> = collection.rest_iter().collect();
assert_eq!(rest_iter, vec![&"second", &"third"]);
```

### Iteration Patterns

```rust
use sugars_collections::ZeroOneOrMany;

let collection = ZeroOneOrMany::many(vec![1, 2, 3]);

// Iterate by reference
for item in &collection {
    println!("Item: {}", item);
}

// Iterate by value (requires T: Clone + 'static)
for item in collection.clone() {
    println!("Owned item: {}", item);
}

// Handle all variants
match &collection {
    ZeroOneOrMany::None => println!("No items to iterate"),
    ZeroOneOrMany::One(item) => println!("Single item: {}", item),
    ZeroOneOrMany::Many(items) => {
        for item in items {
            println!("Item: {}", item);
        }
    }
}
```

## Transformation Operations

### Adding Elements

```rust
use sugars_collections::ZeroOneOrMany;

let empty = ZeroOneOrMany::none();

// Add to empty collection
let with_one = empty.with_pushed(1);
assert_eq!(with_one.len(), 1);

// Add to single element
let with_two = with_one.with_pushed(2);
assert_eq!(with_two.len(), 2);

// Insert at specific position
let with_inserted = with_two.with_inserted(1, 10);
assert_eq!(with_inserted.len(), 3);
// Result: [1, 10, 2]
```

### Mapping Operations

```rust
use sugars_collections::ZeroOneOrMany;

// Map empty collection
let empty: ZeroOneOrMany<i32> = ZeroOneOrMany::none();
let mapped_empty = empty.map(|x| x.to_string());
assert!(mapped_empty.is_empty());

// Map single element
let single = ZeroOneOrMany::one(42);
let mapped_single = single.map(|x| x.to_string());
assert_eq!(mapped_single.first(), Some(&"42".to_string()));

// Map multiple elements
let multiple = ZeroOneOrMany::many(vec![1, 2, 3]);
let mapped_multiple = multiple.map(|x| x * 2);
assert_eq!(mapped_multiple.len(), 3);
```

### Try Map with Error Handling

```rust
use sugars_collections::ZeroOneOrMany;

let numbers = ZeroOneOrMany::many(vec![1, 2, 3]);

// Try map with success
let doubled: Result<ZeroOneOrMany<i32>, &str> = numbers.clone()
    .try_map(|n| if n > 0 { Ok(n * 2) } else { Err("negative number") });
assert!(doubled.is_ok());

// Try map with error
let with_negative = ZeroOneOrMany::many(vec![1, -2, 3]);
let result: Result<ZeroOneOrMany<i32>, &str> = with_negative
    .try_map(|n| if n > 0 { Ok(n * 2) } else { Err("negative number") });
assert!(result.is_err());
```

## JSON Object Syntax

### With Hashbrown HashMap

```rust
use sugars_collections::ZeroOneOrMany;
use hashbrown::HashMap;

let mut map = HashMap::new();
map.insert("key1", "value1");
map.insert("key2", "value2");

// Create from HashMap
let collection: ZeroOneOrMany<(&str, &str)> = ZeroOneOrMany::from_hashmap(map);
assert_eq!(collection.len(), 2);

// Empty HashMap becomes None
let empty_map = HashMap::new();
let empty_collection: ZeroOneOrMany<(&str, &str)> = ZeroOneOrMany::from_hashmap(empty_map);
assert!(empty_collection.is_empty());
```

### With JSON Syntax (requires hashbrown-json feature)

```rust
use sugars_collections::ZeroOneOrMany;

// Using closure syntax for JSON objects
let collection: ZeroOneOrMany<(&str, &str)> = ZeroOneOrMany::from_json(|| {
    let mut map = hashbrown::HashMap::new();
    map.insert("beta", "true");
    map.insert("version", "2.1.0");
    map
});

assert_eq!(collection.len(), 2);
```

## Merging and Combining

### Merging Multiple Collections

```rust
use sugars_collections::ZeroOneOrMany;

let first = ZeroOneOrMany::one(1);
let second = ZeroOneOrMany::many(vec![2, 3]);
let third = ZeroOneOrMany::none();

// Merge collections (requires T: Clone + 'static)
let merged = ZeroOneOrMany::merge(vec![first, second, third]);
assert_eq!(merged.len(), 3);
// Result: [1, 2, 3]
```

### Merging References

```rust
use sugars_collections::ZeroOneOrMany;

let first = ZeroOneOrMany::one(1);
let second = ZeroOneOrMany::many(vec![2, 3]);
let third = ZeroOneOrMany::none();

// Merge references (no Clone required)
let merged_refs = ZeroOneOrMany::merge_refs(vec![&first, &second, &third]);
assert_eq!(merged_refs.len(), 3);
```

## Serialization Support

### JSON Serialization

```rust
use sugars_collections::ZeroOneOrMany;
use serde_json;

// Empty serializes to null
let empty: ZeroOneOrMany<i32> = ZeroOneOrMany::none();
let json = serde_json::to_string(&empty).unwrap();
assert_eq!(json, "null");

// Single element serializes to array
let single = ZeroOneOrMany::one(42);
let json = serde_json::to_string(&single).unwrap();
assert_eq!(json, "[42]");

// Multiple elements serialize to array
let multiple = ZeroOneOrMany::many(vec![1, 2, 3]);
let json = serde_json::to_string(&multiple).unwrap();
assert_eq!(json, "[1,2,3]");
```

### JSON Deserialization

```rust
use sugars_collections::ZeroOneOrMany;
use serde_json;

// From null
let from_null: ZeroOneOrMany<i32> = serde_json::from_str("null").unwrap();
assert!(from_null.is_empty());

// From single value
let from_single: ZeroOneOrMany<i32> = serde_json::from_str("42").unwrap();
assert_eq!(from_single.len(), 1);

// From array
let from_array: ZeroOneOrMany<i32> = serde_json::from_str("[1,2,3]").unwrap();
assert_eq!(from_array.len(), 3);

// From empty array
let from_empty: ZeroOneOrMany<i32> = serde_json::from_str("[]").unwrap();
assert!(from_empty.is_empty());
```

## Common Patterns

### Optional Configuration

```rust
use sugars_collections::ZeroOneOrMany;

#[derive(Debug)]
struct ServerConfig {
    // Optional middleware (zero or more)
    middleware: ZeroOneOrMany<String>,
    // Optional CORS origins (zero or more)
    cors_origins: ZeroOneOrMany<String>,
}

impl ServerConfig {
    fn new() -> Self {
        ServerConfig {
            middleware: ZeroOneOrMany::none(),
            cors_origins: ZeroOneOrMany::none(),
        }
    }
    
    fn with_middleware(mut self, middleware: impl Into<String>) -> Self {
        self.middleware = self.middleware.with_pushed(middleware.into());
        self
    }
    
    fn with_cors_origin(mut self, origin: impl Into<String>) -> Self {
        self.cors_origins = self.cors_origins.with_pushed(origin.into());
        self
    }
    
    fn has_middleware(&self) -> bool {
        !self.middleware.is_empty()
    }
}

// Usage
let config = ServerConfig::new()
    .with_middleware("auth")
    .with_middleware("logging")
    .with_cors_origin("https://example.com");

println!("Has middleware: {}", config.has_middleware());
```

### Flexible Input Handling

```rust
use sugars_collections::ZeroOneOrMany;

fn process_inputs(inputs: ZeroOneOrMany<String>) -> String {
    match inputs {
        ZeroOneOrMany::None => "No inputs provided".to_string(),
        ZeroOneOrMany::One(input) => format!("Processing single input: {}", input),
        ZeroOneOrMany::Many(inputs) => {
            format!("Processing {} inputs: {:?}", inputs.len(), inputs)
        }
    }
}

// Usage with different input types
let result1 = process_inputs(ZeroOneOrMany::none());
let result2 = process_inputs(ZeroOneOrMany::one("input1".to_string()));
let result3 = process_inputs(ZeroOneOrMany::many(vec!["input1".to_string(), "input2".to_string()]));
```

### Builder Pattern with Optional Lists

```rust
use sugars_collections::ZeroOneOrMany;

struct DatabaseQuery {
    table: String,
    conditions: ZeroOneOrMany<String>,
    joins: ZeroOneOrMany<String>,
    order_by: ZeroOneOrMany<String>,
}

impl DatabaseQuery {
    fn new(table: impl Into<String>) -> Self {
        DatabaseQuery {
            table: table.into(),
            conditions: ZeroOneOrMany::none(),
            joins: ZeroOneOrMany::none(),
            order_by: ZeroOneOrMany::none(),
        }
    }
    
    fn where_clause(mut self, condition: impl Into<String>) -> Self {
        self.conditions = self.conditions.with_pushed(condition.into());
        self
    }
    
    fn join(mut self, join: impl Into<String>) -> Self {
        self.joins = self.joins.with_pushed(join.into());
        self
    }
    
    fn order_by(mut self, column: impl Into<String>) -> Self {
        self.order_by = self.order_by.with_pushed(column.into());
        self
    }
    
    fn build_sql(&self) -> String {
        let mut sql = format!("SELECT * FROM {}", self.table);
        
        if !self.joins.is_empty() {
            for join in &self.joins {
                sql.push_str(&format!(" {}", join));
            }
        }
        
        if !self.conditions.is_empty() {
            sql.push_str(" WHERE ");
            let conditions: Vec<String> = self.conditions.iter().cloned().collect();
            sql.push_str(&conditions.join(" AND "));
        }
        
        if !self.order_by.is_empty() {
            sql.push_str(" ORDER BY ");
            let order_columns: Vec<String> = self.order_by.iter().cloned().collect();
            sql.push_str(&order_columns.join(", "));
        }
        
        sql
    }
}

// Usage
let query = DatabaseQuery::new("users")
    .where_clause("age > 18")
    .where_clause("active = true")
    .join("LEFT JOIN profiles ON users.id = profiles.user_id")
    .order_by("created_at DESC")
    .order_by("name ASC");

println!("SQL: {}", query.build_sql());
```

## Best Practices

### 1. Use Pattern Matching for Variant Handling

```rust
use sugars_collections::ZeroOneOrMany;

fn handle_collection(items: ZeroOneOrMany<String>) {
    match items {
        ZeroOneOrMany::None => {
            // Handle empty case explicitly
            println!("No items to process");
        }
        ZeroOneOrMany::One(item) => {
            // Optimize for single item case
            println!("Processing single item: {}", item);
        }
        ZeroOneOrMany::Many(items) => {
            // Handle multiple items
            println!("Processing {} items", items.len());
            for item in items {
                println!("  - {}", item);
            }
        }
    }
}
```

### 2. Leverage Type Safety for Optional Collections

```rust
use sugars_collections::ZeroOneOrMany;

// Good: Express optional nature clearly
struct ApiResponse {
    data: String,
    warnings: ZeroOneOrMany<String>, // May have 0, 1, or many warnings
}

// Bad: Option<Vec<_>> is less clear about intent
struct ApiResponseBad {
    data: String,
    warnings: Option<Vec<String>>, // None vs empty Vec confusion
}
```

### 3. Use Appropriate Creation Methods

```rust
use sugars_collections::ZeroOneOrMany;

// Good: Use specific constructors
let empty = ZeroOneOrMany::none();
let single = ZeroOneOrMany::one("item");
let multiple = ZeroOneOrMany::many(vec!["item1", "item2"]);

// Good: Use from_iter for dynamic creation
let from_iterator: ZeroOneOrMany<i32> = (0..5).collect();
```

### 4. Handle Empty Cases Gracefully

```rust
use sugars_collections::ZeroOneOrMany;

fn process_safely(items: ZeroOneOrMany<String>) -> String {
    if items.is_empty() {
        return "No items to process".to_string();
    }
    
    // Safe to proceed knowing we have items
    format!("Processing {} items", items.len())
}
```

## Examples

### Event Handler System

```rust
use sugars_collections::ZeroOneOrMany;

type EventHandler = Box<dyn Fn(&str) + Send + Sync>;

struct EventBus {
    handlers: std::collections::HashMap<String, ZeroOneOrMany<EventHandler>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus {
            handlers: std::collections::HashMap::new(),
        }
    }
    
    fn subscribe(&mut self, event: String, handler: EventHandler) {
        let current = self.handlers.remove(&event).unwrap_or(ZeroOneOrMany::none());
        self.handlers.insert(event, current.with_pushed(handler));
    }
    
    fn emit(&self, event: &str, data: &str) {
        if let Some(handlers) = self.handlers.get(event) {
            match handlers {
                ZeroOneOrMany::None => {
                    // No handlers, silent
                }
                ZeroOneOrMany::One(handler) => {
                    handler(data);
                }
                ZeroOneOrMany::Many(handlers) => {
                    for handler in handlers {
                        handler(data);
                    }
                }
            }
        }
    }
}

// Usage
let mut bus = EventBus::new();

bus.subscribe("user_login".to_string(), Box::new(|data| {
    println!("Logging: User login - {}", data);
}));

bus.subscribe("user_login".to_string(), Box::new(|data| {
    println!("Analytics: User login - {}", data);
}));

bus.emit("user_login", "user123");
```

### Configuration System

```rust
use sugars_collections::ZeroOneOrMany;
use std::collections::HashMap;

#[derive(Debug)]
struct AppConfig {
    database_urls: ZeroOneOrMany<String>,
    feature_flags: ZeroOneOrMany<String>,
    environment_vars: HashMap<String, String>,
}

impl AppConfig {
    fn new() -> Self {
        AppConfig {
            database_urls: ZeroOneOrMany::none(),
            feature_flags: ZeroOneOrMany::none(),
            environment_vars: HashMap::new(),
        }
    }
    
    fn with_database_url(mut self, url: String) -> Self {
        self.database_urls = self.database_urls.with_pushed(url);
        self
    }
    
    fn with_feature_flag(mut self, flag: String) -> Self {
        self.feature_flags = self.feature_flags.with_pushed(flag);
        self
    }
    
    fn primary_database(&self) -> Option<&String> {
        self.database_urls.first()
    }
    
    fn has_feature(&self, feature: &str) -> bool {
        self.feature_flags.iter().any(|f| f == feature)
    }
    
    fn database_count(&self) -> usize {
        self.database_urls.len()
    }
}

// Usage
let config = AppConfig::new()
    .with_database_url("postgres://localhost/primary".to_string())
    .with_database_url("postgres://localhost/replica".to_string())
    .with_feature_flag("beta_ui".to_string())
    .with_feature_flag("new_auth".to_string());

println!("Primary DB: {:?}", config.primary_database());
println!("Has beta UI: {}", config.has_feature("beta_ui"));
println!("Database count: {}", config.database_count());
```

This guide demonstrates the flexibility and memory efficiency of ZeroOneOrMany in handling collections that can have zero, one, or many elements, providing optimal performance for each case.