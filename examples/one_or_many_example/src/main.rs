//! OneOrMany Examples - Non-empty Collection Type
//!
//! This example demonstrates how to use OneOrMany with:
//! 1. Single element creation
//! 2. Multiple element creation  
//! 3. Transformation operations
//! 4. JSON serialization/deserialization
//! 5. Builder pattern integration
//! 6. Error handling

use serde::{Deserialize, Serialize};
use sugars_collections::OneOrMany;

#[derive(Debug, Serialize, Deserialize)]
struct ServerConfig {
    endpoints: OneOrMany<String>,
    middleware: OneOrMany<String>,
}

#[derive(Debug)]
struct LoadBalancer {
    servers: OneOrMany<String>,
    health_check_interval: u64,
}

impl LoadBalancer {
    fn new(servers: OneOrMany<String>) -> Self {
        LoadBalancer {
            servers,
            health_check_interval: 30,
        }
    }

    fn add_server(self, server: String) -> Self {
        LoadBalancer {
            servers: self.servers.with_pushed(server),
            health_check_interval: self.health_check_interval,
        }
    }

    fn primary_server(&self) -> &String {
        self.servers.first()
    }

    fn all_servers(&self) -> Vec<&String> {
        self.servers.iter().collect()
    }
}

fn main() {
    println!("=== OneOrMany Examples ===\n");

    // Example 1: Single element creation
    println!("1. Single Element Creation:");
    single_element_example();
    println!();

    // Example 2: Multiple element creation
    println!("2. Multiple Element Creation:");
    multiple_element_example();
    println!();

    // Example 3: Transformation operations
    println!("3. Transformation Operations:");
    transformation_example();
    println!();

    // Example 4: JSON serialization
    println!("4. JSON Serialization:");
    json_serialization_example();
    println!();

    // Example 5: Builder pattern
    println!("5. Builder Pattern:");
    builder_pattern_example();
    println!();

    // Example 6: Error handling
    println!("6. Error Handling:");
    error_handling_example();
    println!();

    // Example 7: Merging collections
    println!("7. Merging Collections:");
    merging_example();
    println!();

    // Example 8: Iteration patterns
    println!("8. Iteration Patterns:");
    iteration_example();
    println!();
}

fn single_element_example() {
    // Create from single value
    let single = OneOrMany::one("server1.example.com");
    println!("  Single element: {:?}", single);
    println!("  Length: {}", single.len());
    println!("  First: {:?}", single.first());
    println!("  Is empty: {}", single.is_empty());

    // Using From trait
    let from_value: OneOrMany<i32> = 42.into();
    println!("  From value: {:?}", from_value);
}

fn multiple_element_example() {
    // Create from Vec
    let multiple = OneOrMany::many(vec!["server1", "server2", "server3"]).unwrap();
    println!("  Multiple elements: {:?}", multiple);
    println!("  Length: {}", multiple.len());
    println!("  First: {:?}", multiple.first());

    // Get remaining elements
    let rest = multiple.rest();
    println!("  Rest: {:?}", rest);

    // Try to create from empty Vec (fails)
    match OneOrMany::many(vec![] as Vec<&str>) {
        Ok(_) => println!("  Empty Vec succeeded (unexpected)"),
        Err(e) => println!("  Empty Vec failed as expected: {}", e),
    }
}

fn transformation_example() {
    let original = OneOrMany::one(10);
    println!("  Original: {:?}", original);

    // Add element
    let with_pushed = original.with_pushed(20);
    println!("  With pushed: {:?}", with_pushed);

    // Insert at position
    let with_inserted = with_pushed.with_inserted(1, 15);
    println!("  With inserted: {:?}", with_inserted);

    // Map operation
    let mapped = OneOrMany::many(vec![1, 2, 3]).unwrap().map(|x| x * 2);
    println!("  Mapped (doubled): {:?}", mapped);

    // Try map with error handling
    let numbers = OneOrMany::many(vec![1, 2, 3]).unwrap();
    let result: Result<OneOrMany<i32>, &str> = numbers.try_map(|x| {
        if x > 0 {
            Ok(x * x)
        } else {
            Err("negative number")
        }
    });
    println!("  Try map result: {:?}", result);
}

fn json_serialization_example() {
    // Create server config
    let config = ServerConfig {
        endpoints: OneOrMany::many(vec![
            "https://api.example.com".to_string(),
            "https://api-backup.example.com".to_string(),
        ])
        .unwrap(),
        middleware: OneOrMany::one("auth".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("  Serialized config:\n{}", json);

    // Deserialize from JSON
    let json_str = r#"
    {
        "endpoints": ["https://api.example.com", "https://api-backup.example.com"],
        "middleware": ["auth"]
    }
    "#;

    let deserialized: ServerConfig = serde_json::from_str(json_str).unwrap();
    println!("  Deserialized config: {:?}", deserialized);
}

fn builder_pattern_example() {
    // Create load balancer with single server
    let lb = LoadBalancer::new(OneOrMany::one("server1.example.com".to_string()));
    println!("  Initial load balancer: {:?}", lb);

    // Add more servers
    let lb = lb
        .add_server("server2.example.com".to_string())
        .add_server("server3.example.com".to_string());

    println!("  Final load balancer: {:?}", lb);
    println!("  Primary server: {}", lb.primary_server());
    println!("  All servers: {:?}", lb.all_servers());
}

fn error_handling_example() {
    // Function that validates inputs
    fn validate_servers(servers: Vec<String>) -> Result<OneOrMany<String>, String> {
        if servers.is_empty() {
            return Err("At least one server is required".to_string());
        }

        for server in &servers {
            if !server.contains(".") {
                return Err(format!("Invalid server format: {}", server));
            }
        }

        OneOrMany::many(servers).map_err(|e| e.to_string())
    }

    // Test with valid servers
    let valid_servers = vec!["server1.com".to_string(), "server2.com".to_string()];
    match validate_servers(valid_servers) {
        Ok(servers) => println!("  Valid servers: {:?}", servers),
        Err(e) => println!("  Error: {}", e),
    }

    // Test with empty list
    match validate_servers(vec![]) {
        Ok(servers) => println!("  Empty servers: {:?}", servers),
        Err(e) => println!("  Error: {}", e),
    }

    // Test with invalid format
    let invalid_servers = vec!["server1.com".to_string(), "invalid".to_string()];
    match validate_servers(invalid_servers) {
        Ok(servers) => println!("  Invalid servers: {:?}", servers),
        Err(e) => println!("  Error: {}", e),
    }
}

fn merging_example() {
    let first = OneOrMany::one("group1".to_string());
    let second = OneOrMany::many(vec!["group2".to_string(), "group3".to_string()]).unwrap();
    let third = OneOrMany::one("group4".to_string());

    // Merge collections
    let merged = OneOrMany::merge(vec![first, second, third]).unwrap();
    println!("  Merged collections: {:?}", merged);

    // Merge references
    let ref1 = OneOrMany::one(1);
    let ref2 = OneOrMany::many(vec![2, 3]).unwrap();
    let merged_refs = OneOrMany::merge_refs(vec![&ref1, &ref2]).unwrap();
    println!("  Merged references: {:?}", merged_refs);
}

fn iteration_example() {
    let collection = OneOrMany::many(vec!["apple", "banana", "cherry"]).unwrap();

    // Iterate by reference
    print!("  By reference: ");
    for item in &collection {
        print!("{} ", item);
    }
    println!();

    // Iterate by value (requires Clone)
    print!("  By value: ");
    for item in collection.clone() {
        print!("{} ", item);
    }
    println!();

    // Using iterator methods
    let uppercased: Vec<String> = collection.iter().map(|s| s.to_uppercase()).collect();
    println!("  Uppercased: {:?}", uppercased);

    // From iterator
    let from_iter: OneOrMany<i32> = (1..=5).collect();
    println!("  From iterator: {:?}", from_iter);
}
