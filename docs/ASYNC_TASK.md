# AsyncTask Usage Guide

This guide demonstrates how to use AsyncTask with single and multiple receivers in the cyrup-sugars ecosystem.

## Table of Contents

1. [Overview](#overview)
2. [Single Receiver Usage](#single-receiver-usage)
3. [Multiple Receivers Usage](#multiple-receivers-usage)
4. [ZeroOneOrMany Pattern](#zerooneormany-pattern)
5. [Common Patterns](#common-patterns)
6. [Best Practices](#best-practices)
7. [Examples](#examples)

## Overview

AsyncTask is a concrete async primitive that wraps oneshot channels to provide a Future-based API without boxed futures or async fn. It uses the `ZeroOneOrMany` collection type to handle various receiver scenarios.

### Key Features
- **Concrete return types**: No boxed futures or async fn
- **Flexible receiver handling**: Supports zero, one, or many receivers
- **Channel-based**: Built on tokio oneshot channels
- **NotResult constraint**: Prevents Result types to enforce proper error handling

## Single Receiver Usage

### Basic Single Receiver

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // Create a oneshot channel
    let (tx, rx) = oneshot::channel();
    
    // Create AsyncTask with single receiver
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));
    
    // Send data
    tokio::spawn(async move {
        let _ = tx.send("Hello, World!");
    });
    
    // Await the result
    let result = task.await;
    println!("Received: {}", result);
}
```

### From Future Pattern

```rust
use sugars_async_task::AsyncTask;

async fn some_async_operation() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "Operation completed".to_string()
}

#[tokio::main]
async fn main() {
    // Create AsyncTask from a future
    let task = AsyncTask::from_future(some_async_operation());
    
    let result = task.await;
    println!("Result: {}", result);
}
```

### From Value Pattern

```rust
use sugars_async_task::AsyncTask;

#[tokio::main]
async fn main() {
    // Create AsyncTask from an immediate value
    let task = AsyncTask::from_value(42);
    
    let result = task.await;
    println!("Value: {}", result);
}
```

## Multiple Receivers Usage

### Race Condition - First Result Wins

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // Create multiple channels
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();
    
    // Create AsyncTask with multiple receivers
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));
    
    // Spawn tasks that will send at different times
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        let _ = tx1.send("First");
    });
    
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        let _ = tx2.send("Second");  // This will win
    });
    
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        let _ = tx3.send("Third");
    });
    
    // Gets the first result that arrives
    let result = task.await;
    println!("First result: {}", result);
}
```

### Parallel Processing with Multiple Sources

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

async fn process_data_source_1() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
    "Data from source 1".to_string()
}

async fn process_data_source_2() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "Data from source 2".to_string()
}

async fn process_data_source_3() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    "Data from source 3".to_string()
}

#[tokio::main]
async fn main() {
    // Create channels for each data source
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();
    
    // Spawn tasks for each data source
    tokio::spawn(async move {
        let data = process_data_source_1().await;
        let _ = tx1.send(data);
    });
    
    tokio::spawn(async move {
        let data = process_data_source_2().await;
        let _ = tx2.send(data);
    });
    
    tokio::spawn(async move {
        let data = process_data_source_3().await;
        let _ = tx3.send(data);
    });
    
    // Create AsyncTask that takes the first available result
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));
    
    let result = task.await;
    println!("Fastest source returned: {}", result);
}
```

## ZeroOneOrMany Pattern

### Zero Receivers (Empty)

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;

#[tokio::main]
async fn main() {
    // Create AsyncTask with no receivers (will panic when awaited)
    let task: AsyncTask<String> = AsyncTask::new(ZeroOneOrMany::none());
    
    // This will panic because the channel is closed
    // let result = task.await; // Don't do this!
}
```

### One Receiver

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    
    // Single receiver wrapped in ZeroOneOrMany
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));
    
    tokio::spawn(async move {
        let _ = tx.send("Single result");
    });
    
    let result = task.await;
    println!("Result: {}", result);
}
```

### Many Receivers

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    
    // Multiple receivers
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2]));
    
    tokio::spawn(async move {
        let _ = tx1.send("First");
    });
    
    tokio::spawn(async move {
        let _ = tx2.send("Second");
    });
    
    // Gets the first result
    let result = task.await;
    println!("Result: {}", result);
}
```

## Common Patterns

### Timeout Pattern

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));
    
    // Simulate slow operation
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        let _ = tx.send("Completed");
    });
    
    // Apply timeout
    match timeout(Duration::from_millis(100), task).await {
        Ok(result) => println!("Result: {}", result),
        Err(_) => println!("Operation timed out"),
    }
}
```

### Fallback Pattern

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

async fn primary_service() -> Result<String, &'static str> {
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    Err("Primary service failed")
}

async fn fallback_service() -> Result<String, &'static str> {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    Ok("Fallback service succeeded".to_string())
}

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    
    // Try primary service
    tokio::spawn(async move {
        match primary_service().await {
            Ok(result) => { let _ = tx1.send(result); }
            Err(_) => { /* Don't send anything */ }
        }
    });
    
    // Try fallback service
    tokio::spawn(async move {
        match fallback_service().await {
            Ok(result) => { let _ = tx2.send(result); }
            Err(_) => { /* Don't send anything */ }
        }
    });
    
    // Use the first successful result
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2]));
    let result = task.await;
    println!("Service result: {}", result);
}
```

### Load Balancing Pattern

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

async fn server_1() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(120)).await;
    "Response from server 1".to_string()
}

async fn server_2() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(80)).await;
    "Response from server 2".to_string()
}

async fn server_3() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "Response from server 3".to_string()
}

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();
    
    // Send same request to all servers
    tokio::spawn(async move {
        let response = server_1().await;
        let _ = tx1.send(response);
    });
    
    tokio::spawn(async move {
        let response = server_2().await;
        let _ = tx2.send(response);
    });
    
    tokio::spawn(async move {
        let response = server_3().await;
        let _ = tx3.send(response);
    });
    
    // Take the fastest response
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));
    let result = task.await;
    println!("Fastest server: {}", result);
}
```

## Best Practices

### 1. Proper Error Handling

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

// Good: Handle errors before creating AsyncTask
async fn process_with_error_handling() -> String {
    match risky_operation().await {
        Ok(result) => result,
        Err(_) => "Default value".to_string(),
    }
}

async fn risky_operation() -> Result<String, &'static str> {
    Ok("Success".to_string())
}

#[tokio::main]
async fn main() {
    let task = AsyncTask::from_future(process_with_error_handling());
    let result = task.await;
    println!("Result: {}", result);
}
```

### 2. Channel Management

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    
    // Good: Always handle the sender properly
    let handle = tokio::spawn(async move {
        // Do some work
        let result = "Work completed".to_string();
        
        // Always check if send succeeds
        match tx.send(result) {
            Ok(_) => println!("Result sent successfully"),
            Err(_) => println!("Receiver was dropped"),
        }
    });
    
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));
    let result = task.await;
    println!("Received: {}", result);
    
    // Wait for the spawned task to complete
    let _ = handle.await;
}
```

### 3. Resource Management

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() {
    // Good: Create channels just before use
    let (tx, rx) = oneshot::channel();
    
    // Good: Spawn task immediately after creating channel
    let handle = tokio::spawn(async move {
        let _ = tx.send("Result");
    });
    
    // Good: Create AsyncTask and await immediately
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));
    let result = task.await;
    
    println!("Result: {}", result);
    let _ = handle.await;
}
```

## Examples

### Stream Processing with AsyncTask

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

async fn process_stream_data(data: Vec<i32>) -> i32 {
    data.iter().sum()
}

#[tokio::main]
async fn main() {
    let (tx, rx) = oneshot::channel();
    
    tokio::spawn(async move {
        let data = vec![1, 2, 3, 4, 5];
        let result = process_stream_data(data).await;
        let _ = tx.send(result);
    });
    
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));
    let sum = task.await;
    println!("Sum: {}", sum);
}
```

### Multiple Data Sources

```rust
use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;

async fn fetch_from_database() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "Database data".to_string()
}

async fn fetch_from_cache() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    "Cache data".to_string()
}

async fn fetch_from_api() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    "API data".to_string()
}

#[tokio::main]
async fn main() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();
    
    // Fetch from all sources in parallel
    tokio::spawn(async move {
        let data = fetch_from_database().await;
        let _ = tx1.send(data);
    });
    
    tokio::spawn(async move {
        let data = fetch_from_cache().await;
        let _ = tx2.send(data);
    });
    
    tokio::spawn(async move {
        let data = fetch_from_api().await;
        let _ = tx3.send(data);
    });
    
    // Get the fastest response
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));
    let result = task.await;
    println!("Fastest data source: {}", result);
}
```

This guide demonstrates the flexibility of AsyncTask in handling various async scenarios with concrete types and channel-based communication. The `ZeroOneOrMany` pattern provides a clean way to handle different receiver scenarios while maintaining type safety and performance.