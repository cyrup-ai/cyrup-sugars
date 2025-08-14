//! AsyncTask Examples - Single and Multiple Receivers
//!
//! This example demonstrates how to use AsyncTask with:
//! 1. Single receiver (basic usage)
//! 2. Multiple receivers (race condition - first result wins)
//! 3. From future pattern
//! 4. From value pattern

use sugars_async_task::AsyncTask;
use sugars_collections::ZeroOneOrMany;
use tokio::sync::oneshot;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("=== AsyncTask Examples ===\n");

    // Example 1: Single receiver
    println!("1. Single Receiver Example:");
    single_receiver_example().await;
    println!();

    // Example 2: Multiple receivers (race condition)
    println!("2. Multiple Receivers Example (Race Condition):");
    multiple_receivers_example().await;
    println!();

    // Example 3: From future
    println!("3. From Future Example:");
    from_future_example().await;
    println!();

    // Example 4: From value
    println!("4. From Value Example:");
    from_value_example().await;
    println!();

    // Example 5: Parallel processing
    println!("5. Parallel Processing Example:");
    parallel_processing_example().await;
    println!();

    // Example 6: Timeout pattern
    println!("6. Timeout Pattern Example:");
    timeout_pattern_example().await;
    println!();
}

async fn single_receiver_example() {
    let (tx, rx) = oneshot::channel();

    // Create AsyncTask with single receiver
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));

    // Spawn a task to send data
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        let _ = tx.send("Hello from single receiver!");
    });

    let result = task.await;
    println!("  Received: {}", result);
}

async fn multiple_receivers_example() {
    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();

    // Create AsyncTask with multiple receivers
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));

    // Spawn tasks with different delays
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        let _ = tx1.send("Message from sender 1 (slow)");
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        let _ = tx2.send("Message from sender 2 (fast)"); // This will win
    });

    tokio::spawn(async move {
        sleep(Duration::from_millis(150)).await;
        let _ = tx3.send("Message from sender 3 (medium)");
    });

    let result = task.await;
    println!("  First result: {}", result);
}

async fn from_future_example() {
    async fn compute_value() -> String {
        sleep(Duration::from_millis(100)).await;
        "Computed value from future".to_string()
    }

    let task = AsyncTask::from_future(compute_value());
    let result = task.await;
    println!("  Result: {}", result);
}

async fn from_value_example() {
    let task = AsyncTask::from_value("Immediate value");
    let result = task.await;
    println!("  Result: {}", result);
}

async fn parallel_processing_example() {
    async fn process_data_source(name: &str, delay: u64) -> String {
        sleep(Duration::from_millis(delay)).await;
        format!("Data from {}", name)
    }

    let (tx1, rx1) = oneshot::channel();
    let (tx2, rx2) = oneshot::channel();
    let (tx3, rx3) = oneshot::channel();

    // Spawn parallel processing tasks
    tokio::spawn(async move {
        let data = process_data_source("Database", 120).await;
        let _ = tx1.send(data);
    });

    tokio::spawn(async move {
        let data = process_data_source("Cache", 80).await;
        let _ = tx2.send(data);
    });

    tokio::spawn(async move {
        let data = process_data_source("API", 100).await;
        let _ = tx3.send(data);
    });

    // Get the fastest result
    let task = AsyncTask::new(ZeroOneOrMany::many(vec![rx1, rx2, rx3]));
    let result = task.await;
    println!("  Fastest source: {}", result);
}

async fn timeout_pattern_example() {
    use tokio::time::timeout;

    let (tx, rx) = oneshot::channel();
    let task = AsyncTask::new(ZeroOneOrMany::one(rx));

    // Simulate slow operation
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        let _ = tx.send("Slow operation completed");
    });

    // Apply timeout
    match timeout(Duration::from_millis(100), task).await {
        Ok(result) => println!("  Result: {}", result),
        Err(_) => println!("  Operation timed out"),
    }
}
