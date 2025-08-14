//! ZeroOneOrMany Examples - Flexible Collection Type
//!
//! This example demonstrates how to use ZeroOneOrMany with:
//! 1. All three variants (None, One, Many)
//! 2. Pattern matching for proper handling
//! 3. Transformation operations
//! 4. JSON serialization/deserialization
//! 5. Builder pattern with optional fields
//! 6. Event handling systems

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use sugars_collections::ZeroOneOrMany;

#[derive(Debug, Serialize, Deserialize)]
struct ApiConfig {
    middleware: ZeroOneOrMany<String>,
    cors_origins: ZeroOneOrMany<String>,
    rate_limits: ZeroOneOrMany<u32>,
}

#[derive(Debug)]
struct EventBus {
    listeners: HashMap<String, ZeroOneOrMany<String>>,
}

impl EventBus {
    fn new() -> Self {
        EventBus {
            listeners: HashMap::new(),
        }
    }

    fn add_listener(&mut self, event: String, listener: String) {
        let current = self
            .listeners
            .remove(&event)
            .unwrap_or(ZeroOneOrMany::none());
        self.listeners.insert(event, current.with_pushed(listener));
    }

    fn get_listeners(&self, event: &str) -> ZeroOneOrMany<String> {
        self.listeners
            .get(event)
            .cloned()
            .unwrap_or(ZeroOneOrMany::none())
    }

    fn listener_count(&self, event: &str) -> usize {
        self.get_listeners(event).len()
    }
}

fn main() {
    println!("=== ZeroOneOrMany Examples ===\n");

    // Example 1: All three variants
    println!("1. All Three Variants:");
    variant_examples();
    println!();

    // Example 2: Pattern matching
    println!("2. Pattern Matching:");
    pattern_matching_example();
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

    // Example 6: Event handling
    println!("6. Event Handling:");
    event_handling_example();
    println!();

    // Example 7: Merging collections
    println!("7. Merging Collections:");
    merging_example();
    println!();

    // Example 8: Safe access patterns
    println!("8. Safe Access Patterns:");
    safe_access_example();
    println!();
}

fn variant_examples() {
    // None variant
    let empty: ZeroOneOrMany<String> = ZeroOneOrMany::none();
    println!("  Empty: {:?}", empty);
    println!("  Length: {}", empty.len());
    println!("  Is empty: {}", empty.is_empty());

    // One variant
    let single = ZeroOneOrMany::one("single_item");
    println!("  Single: {:?}", single);
    println!("  Length: {}", single.len());
    println!("  First: {:?}", single.first());

    // Many variant
    let multiple = ZeroOneOrMany::many(vec!["item1", "item2", "item3"]);
    println!("  Multiple: {:?}", multiple);
    println!("  Length: {}", multiple.len());
    println!("  First: {:?}", multiple.first());

    // Empty Vec becomes None
    let empty_vec: ZeroOneOrMany<&str> = ZeroOneOrMany::many(vec![]);
    println!("  Empty Vec: {:?}", empty_vec);
    println!("  Empty Vec is empty: {}", empty_vec.is_empty());
}

fn pattern_matching_example() {
    let collections = [
        ZeroOneOrMany::none(),
        ZeroOneOrMany::one("single"),
        ZeroOneOrMany::many(vec!["first", "second", "third"]),
    ];

    for (i, collection) in collections.iter().enumerate() {
        print!("  Collection {}: ", i + 1);
        match collection {
            ZeroOneOrMany::None => {
                println!("Empty collection");
            }
            ZeroOneOrMany::One(item) => {
                println!("Single item: {}", item);
            }
            ZeroOneOrMany::Many(items) => {
                println!("Multiple items: {:?}", items);
            }
        }
    }
}

fn transformation_example() {
    // Start with empty
    let mut current = ZeroOneOrMany::none();
    println!("  Start: {:?}", current);

    // Add first item (None -> One)
    current = current.with_pushed("item1");
    println!("  After first push: {:?}", current);

    // Add second item (One -> Many)
    current = current.with_pushed("item2");
    println!("  After second push: {:?}", current);

    // Add third item (Many -> Many)
    current = current.with_pushed("item3");
    println!("  After third push: {:?}", current);

    // Insert at position
    current = current.with_inserted(1, "inserted");
    println!("  After insert: {:?}", current);

    // Map operation
    let numbers = ZeroOneOrMany::many(vec![1, 2, 3]);
    let doubled = numbers.map(|x| x * 2);
    println!("  Mapped (doubled): {:?}", doubled);

    // Map empty collection
    let empty_numbers: ZeroOneOrMany<i32> = ZeroOneOrMany::none();
    let doubled_empty = empty_numbers.map(|x| x * 2);
    println!("  Mapped empty: {:?}", doubled_empty);
}

fn json_serialization_example() {
    // Create API config with various states
    let config = ApiConfig {
        middleware: ZeroOneOrMany::many(vec!["auth".to_string(), "cors".to_string()]),
        cors_origins: ZeroOneOrMany::one("https://example.com".to_string()),
        rate_limits: ZeroOneOrMany::none(),
    };

    // Serialize to JSON
    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("  Serialized config:\n{}", json);

    // Test different serialization cases
    let test_cases = vec![
        ("None", ZeroOneOrMany::none() as ZeroOneOrMany<i32>),
        ("One", ZeroOneOrMany::one(42)),
        ("Many", ZeroOneOrMany::many(vec![1, 2, 3])),
    ];

    for (name, case) in test_cases {
        let json = serde_json::to_string(&case).unwrap();
        println!("  {} serializes to: {}", name, json);
    }

    // Deserialize from various JSON formats
    let json_cases = vec![
        ("null", "null"),
        ("single value", "42"),
        ("array", "[1, 2, 3]"),
        ("empty array", "[]"),
    ];

    for (name, json) in json_cases {
        let result: Result<ZeroOneOrMany<i32>, _> = serde_json::from_str(json);
        match result {
            Ok(collection) => println!("  {} deserializes to: {:?}", name, collection),
            Err(e) => println!("  {} failed to deserialize: {}", name, e),
        }
    }
}

fn builder_pattern_example() {
    #[derive(Debug)]
    struct WebServer {
        middleware: ZeroOneOrMany<String>,
        routes: ZeroOneOrMany<String>,
        plugins: ZeroOneOrMany<String>,
    }

    impl WebServer {
        fn new() -> Self {
            WebServer {
                middleware: ZeroOneOrMany::none(),
                routes: ZeroOneOrMany::none(),
                plugins: ZeroOneOrMany::none(),
            }
        }

        fn middleware(mut self, middleware: String) -> Self {
            self.middleware = self.middleware.with_pushed(middleware);
            self
        }

        fn route(mut self, route: String) -> Self {
            self.routes = self.routes.with_pushed(route);
            self
        }

        fn plugin(mut self, plugin: String) -> Self {
            self.plugins = self.plugins.with_pushed(plugin);
            self
        }

        fn has_middleware(&self) -> bool {
            !self.middleware.is_empty()
        }

        fn middleware_count(&self) -> usize {
            self.middleware.len()
        }
    }

    // Build server with no middleware
    let server1 = WebServer::new()
        .route("/api/users".to_string())
        .route("/api/posts".to_string());

    println!("  Server 1: {:?}", server1);
    println!("  Server 1 has middleware: {}", server1.has_middleware());

    // Build server with middleware
    let server2 = WebServer::new()
        .middleware("auth".to_string())
        .middleware("logging".to_string())
        .route("/api/admin".to_string())
        .plugin("metrics".to_string());

    println!("  Server 2: {:?}", server2);
    println!(
        "  Server 2 middleware count: {}",
        server2.middleware_count()
    );
}

fn event_handling_example() {
    let mut bus = EventBus::new();

    // Add listeners to different events
    bus.add_listener("user_login".to_string(), "logger".to_string());
    bus.add_listener("user_login".to_string(), "analytics".to_string());
    bus.add_listener("user_login".to_string(), "notification".to_string());

    bus.add_listener("user_logout".to_string(), "logger".to_string());
    // No listeners for "user_signup" event

    // Check listener counts
    println!(
        "  user_login listeners: {}",
        bus.listener_count("user_login")
    );
    println!(
        "  user_logout listeners: {}",
        bus.listener_count("user_logout")
    );
    println!(
        "  user_signup listeners: {}",
        bus.listener_count("user_signup")
    );

    // Handle events based on listener count
    let events = vec!["user_login", "user_logout", "user_signup"];

    for event in events {
        let listeners = bus.get_listeners(event);
        match listeners {
            ZeroOneOrMany::None => {
                println!("  {}: No listeners", event);
            }
            ZeroOneOrMany::One(listener) => {
                println!("  {}: Single listener - {}", event, listener);
            }
            ZeroOneOrMany::Many(listeners) => {
                println!("  {}: Multiple listeners - {:?}", event, listeners);
            }
        }
    }
}

fn merging_example() {
    let first = ZeroOneOrMany::one("group1".to_string());
    let second = ZeroOneOrMany::many(vec!["group2".to_string(), "group3".to_string()]);
    let third = ZeroOneOrMany::none();
    let fourth = ZeroOneOrMany::one("group4".to_string());

    // Merge all collections
    let merged = ZeroOneOrMany::merge(vec![first, second, third, fourth]);
    println!("  Merged collections: {:?}", merged);

    // Merge references
    let ref1 = ZeroOneOrMany::one(1);
    let ref2 = ZeroOneOrMany::many(vec![2, 3]);
    let ref3 = ZeroOneOrMany::none();
    let merged_refs = ZeroOneOrMany::merge_refs(vec![&ref1, &ref2, &ref3]);
    println!("  Merged references: {:?}", merged_refs);
}

fn safe_access_example() {
    let collections = [
        ZeroOneOrMany::none(),
        ZeroOneOrMany::one("single"),
        ZeroOneOrMany::many(vec!["first", "second", "third"]),
    ];

    for (i, collection) in collections.iter().enumerate() {
        println!("  Collection {}: {:?}", i + 1, collection);

        // Safe first access
        match collection.first() {
            Some(first) => println!("    First: {}", first),
            None => println!("    First: None"),
        }

        // Rest elements
        let rest = collection.rest();
        println!("    Rest: {:?}", rest);

        // Safe iteration
        print!("    Items: ");
        for item in collection {
            print!("{} ", item);
        }
        println!();

        // Length and emptiness
        println!(
            "    Length: {}, Empty: {}",
            collection.len(),
            collection.is_empty()
        );
        println!();
    }
}
