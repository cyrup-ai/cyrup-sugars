//! Tests for macros features

#[cfg(feature = "macros")]
mod macros_tests {
    use cyrup_sugars::macros::collections::*;
    use cyrup_sugars::macros::closures::*;

    #[test]
    fn test_macros_collections_vec_of() {
        let v = vec_of![1, 2, 3, 4, 5];
        assert_eq!(v, vec![1, 2, 3, 4, 5]);
        
        let empty: Vec<i32> = vec_of![];
        assert_eq!(empty, Vec::<i32>::new());
        
        let single = vec_of![42];
        assert_eq!(single, vec![42]);
    }

    #[test]
    fn test_macros_collections_hash_set_of() {
        let set = hash_set_of![1, 2, 3];
        assert_eq!(set.len(), 3);
        assert!(set.contains(&1));
        assert!(set.contains(&2));
        assert!(set.contains(&3));
        
        let empty: std::collections::HashSet<i32> = hash_set_of![];
        assert!(empty.is_empty());
    }

    #[test]
    fn test_macros_collections_hash_map_of() {
        let map = hash_map_of![
            "key1" => "value1",
            "key2" => "value2"
        ];
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("key1"), Some(&"value1"));
        assert_eq!(map.get("key2"), Some(&"value2"));
        
        let empty: std::collections::HashMap<&str, &str> = hash_map_of![];
        assert!(empty.is_empty());
    }

    #[test]
    fn test_macros_closures_capture() {
        let x = 42;
        let closure = capture!(x, |y| x + y);
        assert_eq!(closure(8), 50);
        
        let s = "hello".to_string();
        let closure = capture!(s, |suffix| format!("{} {}", s, suffix));
        assert_eq!(closure("world"), "hello world");
    }

    #[test]
    fn test_macros_closures_clone_capture() {
        let x = vec![1, 2, 3];
        let closure = clone_capture!(x, |item| {
            let mut result = x.clone();
            result.push(item);
            result
        });
        
        let result = closure(4);
        assert_eq!(result, vec![1, 2, 3, 4]);
        
        // Original should be unchanged
        assert_eq!(x, vec![1, 2, 3]);
    }

    #[test]
    fn test_macros_closures_move_capture() {
        let x = String::from("hello");
        let closure = move_capture!(x, |suffix| format!("{} {}", x, suffix));
        
        let result = closure(" world");
        assert_eq!(result, "hello  world");
        
        // x is moved, so this would not compile:
        // println!("{}", x);
    }
}

#[cfg(all(feature = "macros", feature = "hashbrown"))]
mod macros_hashbrown_tests {
    use cyrup_sugars::macros::hashbrown::*;
    use hashbrown::{HashMap, HashSet};

    #[test]
    fn test_macros_hashbrown_hash_map() {
        let map = hash_map! {
            "key1" => "value1",
            "key2" => "value2",
            "key3" => "value3"
        };
        
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("key1"), Some(&"value1"));
        assert_eq!(map.get("key2"), Some(&"value2"));
        assert_eq!(map.get("key3"), Some(&"value3"));
    }

    #[test]
    fn test_macros_hashbrown_hash_set() {
        let set = hash_set! { 1, 2, 3, 4, 5 };
        
        assert_eq!(set.len(), 5);
        assert!(set.contains(&1));
        assert!(set.contains(&3));
        assert!(set.contains(&5));
        assert!(!set.contains(&6));
    }

    #[test]
    fn test_macros_hashbrown_hash_map_fn() {
        let map_fn = hash_map_fn! {
            "api_key" => "secret123",
            "endpoint" => "https://api.example.com",
            "timeout" => "30"
        };
        
        let map = map_fn();
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("api_key"), Some(&"secret123"));
        assert_eq!(map.get("endpoint"), Some(&"https://api.example.com"));
        assert_eq!(map.get("timeout"), Some(&"30"));
    }

    #[test]
    fn test_macros_hashbrown_hash_set_fn() {
        let set_fn = hash_set_fn! { "apple", "banana", "cherry" };
        
        let set = set_fn();
        assert_eq!(set.len(), 3);
        assert!(set.contains("apple"));
        assert!(set.contains("banana"));
        assert!(set.contains("cherry"));
    }
}

#[cfg(all(feature = "macros", feature = "collections"))]
mod macros_collections_integration_tests {
    use cyrup_sugars::macros::collections::*;
    use cyrup_sugars::collections::{OneOrMany, ZeroOneOrMany};

    #[test]
    fn test_macros_collections_with_zero_one_or_many() {
        let items = vec_of![1, 2, 3, 4, 5];
        let collection = ZeroOneOrMany::many(items);
        assert_eq!(collection.len(), 5);
        assert_eq!(collection.first(), Some(&1));
    }

    #[test]
    fn test_macros_collections_with_one_or_many() {
        let items = vec_of![10, 20, 30];
        let collection = OneOrMany::many(items).unwrap();
        assert_eq!(collection.len(), 3);
        assert_eq!(collection.first(), &10);
    }

    #[test]
    fn test_macros_collections_empty_with_zero_one_or_many() {
        let items: Vec<i32> = vec_of![];
        let collection = ZeroOneOrMany::many(items);
        assert!(collection.is_empty());
        assert_eq!(collection.len(), 0);
    }
}