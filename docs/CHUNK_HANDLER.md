# ChunkHandler Pattern Implementation Guide

A complete, step-by-step guide to implementing the ChunkHandler pattern for streaming Result handling in Rust builders.

## Table of Contents

1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Step 1: Define the MessageChunk Trait](#step-1-define-the-messagechunk-trait)
4. [Step 2: Define the ChunkHandler Trait](#step-2-define-the-chunkhandler-trait)
5. [Step 3: Create Your Chunk Type](#step-3-create-your-chunk-type)
6. [Step 4: Implement MessageChunk](#step-4-implement-messagechunk)
7. [Step 5: Create Your Builder](#step-5-create-your-builder)
8. [Step 6: Implement ChunkHandler](#step-6-implement-chunkhandler)
9. [Step 7: Use the Pattern](#step-7-use-the-pattern)
10. [Complete Working Example](#complete-working-example)
11. [Common Patterns](#common-patterns)
12. [Troubleshooting](#troubleshooting)

## Overview

The ChunkHandler pattern solves a fundamental problem in streaming APIs: handling `Result<T, E>` streams where you need to:
- Convert errors to a success type (for continuous streaming)
- Track which chunks represent errors
- Provide a clean builder API for error handling

**Key Benefits:**
- Single method handles both success and error cases
- Type-safe error tracking
- Clean, chainable builder pattern
- No boxing or dynamic dispatch required

## Prerequisites

- Rust 1.70+
- Basic understanding of traits and generics
- Familiarity with builder patterns

Add to your `Cargo.toml`:
```toml
[dependencies]
cyrup_sugars = { version = "0.2", features = ["builders"] }
```

## Step 1: Define the MessageChunk Trait

First, create a trait that allows types to represent both success and error states:

**File: `src/traits.rs`**

```rust
/// Trait for types that can represent both success and error states
pub trait MessageChunk: Sized {
    /// Create an error chunk from an error string
    fn bad_chunk(error: String) -> Self;
    
    /// Get the error message if this is an error chunk
    fn error(&self) -> Option<&str>;
    
    /// Check if this chunk represents an error
    fn is_error(&self) -> bool {
        self.error().is_some()
    }
}
```

**Key Points:**
- `bad_chunk` is a constructor for error states
- `error()` allows checking if a chunk is an error
- `is_error()` provides convenient boolean checking
- The trait is `Sized` for zero-cost abstractions

## Step 2: Define the ChunkHandler Trait

Create the trait that builders will implement:

**File: `src/traits.rs` (continued)**

```rust
/// Trait for handling streaming Results
pub trait ChunkHandler<T, E = String>: Sized 
where
    T: MessageChunk,
{
    /// Handle a Result<T, E> by unwrapping to T
    /// 
    /// The handler function converts both Ok and Err to T
    fn on_chunk<F>(self, handler: F) -> Self
    where
        F: Fn(Result<T, E>) -> T + Send + Sync + 'static;
}
```

**Key Points:**
- Generic over chunk type `T` and error type `E`
- Default error type is `String` for convenience
- Handler must be `Send + Sync` for async contexts
- Returns `Self` for method chaining

## Step 3: Create Your Chunk Type

Define a concrete type that will flow through your stream:

**File: `src/types.rs`**

```rust
use crate::traits::MessageChunk;

/// A chunk of conversation data
#[derive(Debug, Clone)]
pub struct ConversationChunk {
    pub content: String,
    pub metadata: Option<String>,
    // Private field to track errors
    error: Option<String>,
}

impl ConversationChunk {
    /// Create a new success chunk
    pub fn new(content: String) -> Self {
        ConversationChunk {
            content,
            metadata: None,
            error: None,
        }
    }
    
    /// Add metadata to the chunk
    pub fn with_metadata(mut self, metadata: String) -> Self {
        self.metadata = Some(metadata);
        self
    }
}
```

**Key Points:**
- Keep error field private for encapsulation
- Provide constructors for normal use cases
- Use builder methods for optional fields

## Step 4: Implement MessageChunk

Implement the trait for your chunk type:

**File: `src/types.rs` (continued)**

```rust
impl MessageChunk for ConversationChunk {
    fn bad_chunk(error: String) -> Self {
        ConversationChunk {
            // You decide how to represent errors
            content: format!("[ERROR] {}", error),
            metadata: Some("error".to_string()),
            error: Some(error),
        }
    }
    
    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
```

**Implementation Strategies:**
1. **Visible Error**: Include error in content for debugging
2. **Silent Error**: Keep content empty, only set error field
3. **Typed Error**: Use an enum for content with Error variant

## Step 5: Create Your Builder

Create a builder that will use the ChunkHandler trait:

**File: `src/builder.rs`**

```rust
use crate::traits::{MessageChunk, ChunkHandler};
use crate::types::ConversationChunk;

pub struct StreamBuilder {
    // Your configuration fields
    endpoint: String,
    timeout: Option<u64>,
    
    // Handler storage - note the type signature
    chunk_handler: Option<Box<dyn Fn(Result<ConversationChunk, String>) -> ConversationChunk + Send + Sync>>,
}

impl StreamBuilder {
    pub fn new(endpoint: String) -> Self {
        StreamBuilder {
            endpoint,
            timeout: None,
            chunk_handler: None,
        }
    }
    
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = Some(seconds);
        self
    }
    
    // Other builder methods...
}
```

**Key Design Decisions:**
- Store handler as boxed trait object for flexibility
- Make handler optional (use default if not set)
- Include Send + Sync for thread safety

## Step 6: Implement ChunkHandler

Implement the trait for your builder:

**File: `src/builder.rs` (continued)**

```rust
impl ChunkHandler<ConversationChunk, String> for StreamBuilder {
    fn on_chunk<F>(mut self, handler: F) -> Self
    where
        F: Fn(Result<ConversationChunk, String>) -> ConversationChunk + Send + Sync + 'static,
    {
        self.chunk_handler = Some(Box::new(handler));
        self
    }
}
```

**That's it!** The trait provides the method, you just store the handler.

## Step 7: Use the Pattern

Now use your builder with the ChunkHandler pattern:

**File: `src/main.rs`**

```rust
use your_crate::{StreamBuilder, ConversationChunk, ChunkHandler, MessageChunk};

#[tokio::main]
async fn main() {
    let stream = StreamBuilder::new("https://api.example.com".into())
        .timeout(30)
        .on_chunk(|result| match result {
            Ok(chunk) => {
                println!("Received: {}", chunk.content);
                chunk
            },
            Err(error) => {
                eprintln!("Error: {}", error);
                ConversationChunk::bad_chunk(error)
            }
        })
        .build();
    
    // Use the stream...
    process_stream(stream).await;
}

async fn process_stream(builder: StreamBuilder) {
    // Your async stream processing
    // The handler will be called for each chunk
}
```

## Complete Working Example

Here's a full, runnable example:

**File: `examples/chunk_handler_demo.rs`**

```rust
// Import from cyrup_sugars or define locally
use cyrup_sugars::prelude::*;

// 1. Define your chunk type
#[derive(Debug, Clone)]
struct DataChunk {
    data: Vec<u8>,
    timestamp: u64,
    error: Option<String>,
}

// 2. Implement MessageChunk
impl MessageChunk for DataChunk {
    fn bad_chunk(error: String) -> Self {
        DataChunk {
            data: vec![],
            timestamp: 0,
            error: Some(error),
        }
    }
    
    fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}

// 3. Create your builder
struct DataStreamBuilder {
    source: String,
    buffer_size: usize,
    handler: Option<Box<dyn Fn(Result<DataChunk, String>) -> DataChunk + Send + Sync>>,
}

impl DataStreamBuilder {
    fn new(source: String) -> Self {
        DataStreamBuilder {
            source,
            buffer_size: 1024,
            handler: None,
        }
    }
    
    fn buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }
    
    async fn stream(self) -> DataStream {
        DataStream {
            builder: self,
            current_position: 0,
        }
    }
}

// 4. Implement ChunkHandler
impl ChunkHandler<DataChunk, String> for DataStreamBuilder {
    fn on_chunk<F>(mut self, handler: F) -> Self
    where
        F: Fn(Result<DataChunk, String>) -> DataChunk + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }
}

// 5. Create a stream type that uses the handler
struct DataStream {
    builder: DataStreamBuilder,
    current_position: usize,
}

impl DataStream {
    async fn next(&mut self) -> Option<DataChunk> {
        // Simulate getting data
        let result = if self.current_position < 10 {
            Ok(DataChunk {
                data: vec![self.current_position as u8],
                timestamp: self.current_position as u64,
                error: None,
            })
        } else if self.current_position == 10 {
            Err("Simulated error".to_string())
        } else {
            return None;
        };
        
        self.current_position += 1;
        
        // Use the handler if set, otherwise default behavior
        let chunk = if let Some(ref handler) = self.builder.handler {
            handler(result)
        } else {
            result.unwrap_or_else(|e| DataChunk::bad_chunk(e))
        };
        
        Some(chunk)
    }
}

// 6. Use it!
#[tokio::main]
async fn main() {
    let mut stream = DataStreamBuilder::new("data://source".into())
        .buffer_size(2048)
        .on_chunk(|result| match result {
            Ok(chunk) => {
                println!("Data chunk: {} bytes at time {}", 
                         chunk.data.len(), chunk.timestamp);
                chunk
            },
            Err(e) => {
                println!("Error occurred: {}", e);
                DataChunk::bad_chunk(e)
            }
        })
        .stream()
        .await;
    
    // Process the stream
    while let Some(chunk) = stream.next().await {
        if chunk.is_error() {
            println!("Processing error chunk: {:?}", chunk.error());
        } else {
            println!("Processing data: {:?}", chunk.data);
        }
    }
}
```

## Common Patterns

### Pattern 1: Simple Error Logging

```rust
.on_chunk(|result| result.unwrap_or_else(|e| {
    eprintln!("Error: {}", e);
    T::bad_chunk(e)
}))
```

### Pattern 2: Error Recovery

```rust
.on_chunk(|result| match result {
    Ok(chunk) => chunk,
    Err(e) if e.contains("timeout") => {
        // Retry logic or fallback
        T::bad_chunk("Timeout - retrying".into())
    },
    Err(e) => T::bad_chunk(e)
})
```

### Pattern 3: Metrics Collection

```rust
.on_chunk(|result| {
    match &result {
        Ok(_) => metrics.success_count.increment(),
        Err(_) => metrics.error_count.increment(),
    }
    result.unwrap_or_else(|e| T::bad_chunk(e))
})
```

### Pattern 4: Transform Errors

```rust
.on_chunk(|result| result.map_err(|e| {
    format!("Stream error at {}: {}", chrono::Utc::now(), e)
}).unwrap_or_else(|e| T::bad_chunk(e)))
```

## Troubleshooting

### Issue: "trait bound not satisfied"

**Problem:**
```
error[E0277]: the trait bound `MyChunk: MessageChunk` is not satisfied
```

**Solution:** Ensure you've implemented MessageChunk for your chunk type:
```rust
impl MessageChunk for MyChunk {
    fn bad_chunk(error: String) -> Self { /* ... */ }
    fn error(&self) -> Option<&str> { /* ... */ }
}
```

### Issue: "cannot move out of captured variable"

**Problem:** Handler closure tries to move a value

**Solution:** Clone values or use Arc for sharing:
```rust
let shared_state = Arc::new(state);
let state_clone = shared_state.clone();

.on_chunk(move |result| {
    // Use state_clone here
    match result {
        Ok(chunk) => chunk,
        Err(e) => {
            state_clone.log_error(&e);
            T::bad_chunk(e)
        }
    }
})
```

### Issue: Handler not being called

**Problem:** Stream doesn't use the stored handler

**Solution:** Ensure your stream implementation calls the handler:
```rust
// In your stream's next() or poll() method:
let chunk = if let Some(ref handler) = self.handler {
    handler(result)
} else {
    // Default behavior
    result.unwrap_or_else(|e| T::bad_chunk(e))
};
```

### Issue: Lifetime errors with handler

**Problem:** 
```
error[E0310]: the parameter type `F` may not live long enough
```

**Solution:** Add 'static bound to handler:
```rust
fn on_chunk<F>(self, handler: F) -> Self
where
    F: Fn(Result<T, E>) -> T + Send + Sync + 'static,  // Note 'static
```

## Advanced Topics

### Custom Error Types

Use any error type, not just String:

```rust
impl ChunkHandler<DataChunk, MyError> for MyBuilder {
    fn on_chunk<F>(mut self, handler: F) -> Self
    where
        F: Fn(Result<DataChunk, MyError>) -> DataChunk + Send + Sync + 'static,
    {
        self.handler = Some(Box::new(handler));
        self
    }
}
```

### Multiple Handlers

Chain multiple handlers by storing them in a Vec:

```rust
struct MultiHandlerBuilder {
    handlers: Vec<Box<dyn Fn(Result<Chunk, Error>) -> Chunk + Send + Sync>>,
}

impl MultiHandlerBuilder {
    fn add_handler<F>(&mut self, handler: F) 
    where
        F: Fn(Result<Chunk, Error>) -> Chunk + Send + Sync + 'static,
    {
        self.handlers.push(Box::new(handler));
    }
    
    fn process(&self, result: Result<Chunk, Error>) -> Chunk {
        self.handlers.iter().fold(result, |acc, handler| {
            Ok(handler(acc))
        }).unwrap_or_else(|e| Chunk::bad_chunk(e))
    }
}
```

### Integration with async-stream

```rust
use async_stream::stream;

impl DataStream {
    fn into_stream(self) -> impl Stream<Item = DataChunk> {
        let handler = self.builder.handler;
        
        stream! {
            for i in 0..10 {
                let result = fetch_chunk(i).await;
                let chunk = if let Some(ref h) = handler {
                    h(result)
                } else {
                    result.unwrap_or_else(|e| DataChunk::bad_chunk(e))
                };
                yield chunk;
            }
        }
    }
}
```

## Summary

The ChunkHandler pattern provides:

1. **Type Safety**: Errors are converted to success types at compile time
2. **Flexibility**: Handle errors however you want in the closure
3. **Simplicity**: One method, one concern
4. **Performance**: Zero-cost abstraction with no runtime overhead
5. **Composability**: Works with any builder pattern

By following this guide, you can implement robust error handling for any streaming API in Rust.