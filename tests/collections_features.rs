//! Tests for collections feature and related functionality

#[cfg(feature = "collections")]
mod collections_tests {
    use cyrup_sugars::{
        ByteSize, ByteSizeExt, OneOrMany, ZeroOneOrMany
    };

    #[test]
    fn test_collections_byte_size_basic() {
        let size = ByteSize::bytes(1024);
        assert_eq!(size.as_bytes(), 1024);
        
        let mb = ByteSize::megabytes(1);
        assert_eq!(mb.as_bytes(), 1024 * 1024);
    }

    #[test] 
    fn test_collections_byte_size_ext() {
        assert_eq!(1u64.kb().as_bytes(), 1024);
        assert_eq!(2u64.mb().as_bytes(), 2 * 1024 * 1024);
        assert_eq!(512u64.bytes().as_bytes(), 512);
    }

    #[test]
    fn test_collections_zero_one_or_many_basic() {
        let none = ZeroOneOrMany::<i32>::none();
        assert_eq!(none.len(), 0);
        assert!(none.is_empty());
        assert_eq!(none.first(), None);
        
        let one = ZeroOneOrMany::one(42);
        assert_eq!(one.len(), 1);
        assert!(!one.is_empty());
        assert_eq!(one.first(), Some(&42));
        
        let many = ZeroOneOrMany::many(vec![1, 2, 3]);
        assert_eq!(many.len(), 3);
        assert!(!many.is_empty());
        assert_eq!(many.first(), Some(&1));
    }

    #[test]
    fn test_collections_zero_one_or_many_with_pushed() {
        let none = ZeroOneOrMany::<i32>::none();
        let one = none.with_pushed(42);
        assert_eq!(one.len(), 1);
        assert_eq!(one.first(), Some(&42));
        
        let two = one.with_pushed(43);
        assert_eq!(two.len(), 2);
        assert_eq!(two.first(), Some(&42));
    }

    #[test]
    fn test_collections_zero_one_or_many_map() {
        let many = ZeroOneOrMany::many(vec![1, 2, 3]);
        let doubled = many.map(|x| x * 2);
        assert_eq!(doubled.len(), 3);
        let values: Vec<i32> = doubled.into();
        assert_eq!(values, vec![2, 4, 6]);
    }

    #[test]
    fn test_collections_one_or_many_basic() {
        let one = OneOrMany::one(42);
        assert_eq!(one.len(), 1);
        assert_eq!(one.first(), &42);
        
        let many = OneOrMany::many(vec![1, 2, 3]).expect("test data");
        assert_eq!(many.len(), 3);
        assert_eq!(many.first(), &1);
        
        // Test empty fails
        let empty_result: Result<OneOrMany<i32>, _> = OneOrMany::many(vec![]);
        assert!(empty_result.is_err());
    }

    #[test]
    fn test_collections_one_or_many_with_pushed() {
        let one = OneOrMany::one(42);
        let two = one.with_pushed(43);
        assert_eq!(two.len(), 2);
        assert_eq!(two.first(), &42);
    }

    #[test]
    fn test_collections_one_or_many_map() {
        let many = OneOrMany::many(vec![1, 2, 3]).expect("test data");
        let doubled = many.map(|x| x * 2);
        assert_eq!(doubled.len(), 3);
        let values: Vec<i32> = doubled.into();
        assert_eq!(values, vec![2, 4, 6]);
    }

    #[test]
    fn test_collections_conversions() {
        // From single value
        let from_val: OneOrMany<i32> = 42.into();
        assert_eq!(from_val.len(), 1);
        
        let from_val: ZeroOneOrMany<i32> = 42.into();
        assert_eq!(from_val.len(), 1);
        
        // From Vec
        let vec = vec![1, 2, 3];
        let from_vec: ZeroOneOrMany<i32> = vec.into();
        assert_eq!(from_vec.len(), 3);
        
        // To Vec
        let many = ZeroOneOrMany::many(vec![1, 2, 3]);
        let back_to_vec: Vec<i32> = many.into();
        assert_eq!(back_to_vec, vec![1, 2, 3]);
    }

    #[test]
    fn test_collections_iter() {
        let many = ZeroOneOrMany::many(vec![1, 2, 3]);
        let collected: Vec<&i32> = many.iter().collect();
        assert_eq!(collected, vec![&1, &2, &3]);
        
        let one_many = OneOrMany::many(vec![4, 5, 6]).expect("test data");
        let collected: Vec<&i32> = one_many.iter().collect();
        assert_eq!(collected, vec![&4, &5, &6]);
    }

    #[test]
    fn test_collections_rest() {
        let many = ZeroOneOrMany::many(vec![1, 2, 3]);
        let rest = many.rest();
        assert_eq!(rest, vec![&2, &3]);
        
        let one = ZeroOneOrMany::one(42);
        let rest = one.rest();
        assert_eq!(rest, Vec::<&i32>::new());
        
        let none = ZeroOneOrMany::<i32>::none();
        let rest = none.rest();
        assert_eq!(rest, Vec::<&i32>::new());
    }
}

#[cfg(all(feature = "collections", feature = "serde"))]
mod collections_serde_tests {
    use cyrup_sugars::{OneOrMany, ZeroOneOrMany};
    use serde_json;

    #[test]
    fn test_collections_serde_zero_one_or_many() {
        // Test None serialization
        let none = ZeroOneOrMany::<i32>::none();
        let json = serde_json::to_string(&none).unwrap();
        assert_eq!(json, "[]");
        
        // Test One serialization
        let one = ZeroOneOrMany::one(42);
        let json = serde_json::to_string(&one).unwrap();
        assert_eq!(json, "[42]");
        
        // Test Many serialization
        let many = ZeroOneOrMany::many(vec![1, 2, 3]);
        let json = serde_json::to_string(&many).unwrap();
        assert_eq!(json, "[1,2,3]");
    }

    #[test]
    fn test_collections_serde_zero_one_or_many_deserialization() {
        // From null
        let from_null: ZeroOneOrMany<i32> = serde_json::from_str("null").unwrap();
        assert!(from_null.is_empty());
        
        // From empty array
        let from_empty: ZeroOneOrMany<i32> = serde_json::from_str("[]").unwrap();
        assert!(from_empty.is_empty());
        
        // From single value in array
        let from_array: ZeroOneOrMany<i32> = serde_json::from_str("[42]").unwrap();
        assert_eq!(from_array.len(), 1);
        assert_eq!(from_array.first(), Some(&42));
        
        // From multiple values
        let from_multi: ZeroOneOrMany<i32> = serde_json::from_str("[1,2,3]").unwrap();
        assert_eq!(from_multi.len(), 3);
        assert_eq!(from_multi.first(), Some(&1));
    }

    #[test]
    fn test_collections_serde_one_or_many() {
        // Test One serialization
        let one = OneOrMany::one(42);
        let json = serde_json::to_string(&one).unwrap();
        assert_eq!(json, "[42]");
        
        // Test Many serialization
        let many = OneOrMany::many(vec![1, 2, 3]).expect("test data");
        let json = serde_json::to_string(&many).unwrap();
        assert_eq!(json, "[1,2,3]");
    }

    #[test]
    fn test_collections_serde_one_or_many_deserialization() {
        // From single value in array
        let from_array: OneOrMany<i32> = serde_json::from_str("[42]").unwrap();
        assert_eq!(from_array.len(), 1);
        assert_eq!(from_array.first(), &42);
        
        // From multiple values
        let from_multi: OneOrMany<i32> = serde_json::from_str("[1,2,3]").unwrap();
        assert_eq!(from_multi.len(), 3);
        assert_eq!(from_multi.first(), &1);
        
        // Empty array should fail
        let empty_result: Result<OneOrMany<i32>, _> = serde_json::from_str("[]");
        assert!(empty_result.is_err());
    }
}

#[cfg(all(feature = "collections", feature = "hashbrown-json"))]
mod collections_hashbrown_json_tests {
    use cyrup_sugars::{OneOrMany, ZeroOneOrMany};
    use hashbrown::HashMap;

    #[test]
    fn test_collections_hashbrown_json_zero_one_or_many() {
        let mut map = HashMap::new();
        map.insert("key1", "value1");
        map.insert("key2", "value2");
        
        let collection = ZeroOneOrMany::from_hashmap(map);
        assert_eq!(collection.len(), 2);
        
        // Test empty map
        let empty_map = HashMap::new();
        let empty_collection = ZeroOneOrMany::from_hashmap(empty_map);
        assert!(empty_collection.is_empty());
    }

    #[test]
    fn test_collections_hashbrown_json_one_or_many() {
        let mut map = HashMap::new();
        map.insert("key1", "value1");
        map.insert("key2", "value2");
        
        let collection = OneOrMany::from_hashmap(map).expect("test data");
        assert_eq!(collection.len(), 2);
        
        // Test empty map fails
        let empty_map = HashMap::new();
        let empty_result = OneOrMany::from_hashmap(empty_map);
        assert!(empty_result.is_err());
    }

    #[test]
    fn test_collections_hashbrown_json_closure() {
        let collection = ZeroOneOrMany::from_json(|| {
            let mut map = HashMap::new();
            map.insert("beta", "true");
            map.insert("version", "2.1.0");
            map
        });
        assert_eq!(collection.len(), 2);
        
        let one_collection = OneOrMany::from_json(|| {
            let mut map = HashMap::new();
            map.insert("single", "value");
            map
        }).expect("test data");
        assert_eq!(one_collection.len(), 1);
    }
}