//! Tests for feature combinations and edge cases

#[cfg(all(feature = "collections", feature = "hashbrown-json", feature = "serde"))]
mod full_features_tests {
    use cyrup_sugars::{OneOrMany, ZeroOneOrMany};
    use cyrup_sugars::macros::hashbrown::*;
    use serde_json;
    use hashbrown::HashMap;

    #[test]
    fn test_full_features_hashbrown_serde_integration() {
        let map = hash_map! {
            "servers" => ZeroOneOrMany::many(vec!["api.com", "db.com"]),
            "endpoints" => OneOrMany::one("primary.api.com")
        };
        
        // This should serialize properly
        let json = serde_json::to_string_pretty(&map).unwrap();
        assert!(json.contains("api.com"));
        assert!(json.contains("primary.api.com"));
        
        // Test deserialization
        let roundtrip: HashMap<String, ZeroOneOrMany<String>> = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.len(), 2);
    }

    #[test]
    fn test_full_features_collection_from_hashmap_with_serde() {
        let collection = ZeroOneOrMany::from_json(|| {
            let mut map = HashMap::new();
            map.insert("key1", "value1");
            map.insert("key2", "value2");
            map
        });
        
        // Serialize the collection
        let json = serde_json::to_string(&collection).unwrap();
        
        // Deserialize back
        let deserialized: ZeroOneOrMany<(String, String)> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 2);
    }

    #[test]
    fn test_full_features_one_or_many_hashbrown_serde() {
        let collection = OneOrMany::from_json(|| {
            let mut map = HashMap::new();
            map.insert("single", "value");
            map
        }).unwrap();
        
        let json = serde_json::to_string(&collection).unwrap();
        let deserialized: OneOrMany<(String, String)> = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized.first().0, "single");
        assert_eq!(deserialized.first().1, "value");
    }
}

#[cfg(all(feature = "async", feature = "collections"))]
mod async_collections_tests {
    use cyrup_sugars::{AsyncTask, OneOrMany, ZeroOneOrMany};

    #[tokio::test]
    async fn test_async_collections_integration() {
        let items = vec![1, 2, 3, 4, 5];
        let collection = ZeroOneOrMany::many(items);
        let task = AsyncTask::from_value(collection);
        
        let result = task.await;
        assert_eq!(result.len(), 5);
        
        // Use proper closure syntax
        let multiplier = 2;
        let closure = move |x| x * multiplier;
        let doubled = result.map(closure);
        
        let values: Vec<i32> = doubled.into();
        assert_eq!(values, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_async_one_or_many() {
        let items = vec![10, 20, 30];
        let collection = OneOrMany::many(items).unwrap();
        let task = AsyncTask::from_value(collection);
        
        let result = task.await;
        assert_eq!(result.len(), 3);
        assert_eq!(result.first(), &10);
    }
}

#[cfg(all(feature = "gix-interop", feature = "collections"))]
mod gix_collections_tests {
    use cyrup_sugars::external::gix_hashtable::*;
    use cyrup_sugars::{OneOrMany, ZeroOneOrMany};
    use gix_hash::{ObjectId, Kind};

    #[test]
    fn test_gix_collections_integration() {
        let oids = vec![
            ObjectId::empty_tree(Kind::Sha1),
            ObjectId::empty_blob(Kind::Sha1),
            ObjectId::empty_tree(Kind::Sha256)
        ];
        
        let collection = ZeroOneOrMany::many(oids);
        assert_eq!(collection.len(), 3);
        
        // Create a map using the collection
        let mut map = ObjectIdMap::new();
        for (i, oid) in collection.iter().enumerate() {
            map.insert(*oid, format!("object_{}", i));
        }
        
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_gix_one_or_many() {
        let oids = vec![
            ObjectId::empty_tree(Kind::Sha1),
            ObjectId::empty_blob(Kind::Sha1)
        ];
        
        let collection = OneOrMany::many(oids).unwrap();
        assert_eq!(collection.len(), 2);
        
        let mut set = ObjectIdSet::new();
        for oid in collection.iter() {
            set.insert(*oid);
        }
        
        assert_eq!(set.len(), 2);
    }
}

// Tests for no-default-features scenarios
#[cfg(not(feature = "std"))]
mod minimal_tests {
    // When std is not available, very limited functionality should work
    
    #[test]
    fn test_minimal_basic() {
        // Test that basic types are available
        let _x: i32 = 42;
        assert_eq!(_x, 42);
    }
}

#[cfg(all(feature = "std", not(feature = "collections")))]
mod std_only_tests {
    #[test]
    fn test_std_only_basic() {
        // Test std types without collections
        let vec = vec![1, 2, 3];
        assert_eq!(vec.len(), 3);
    }
}

#[cfg(all(feature = "collections", not(feature = "serde")))]
mod collections_no_serde_tests {
    use cyrup_sugars::{OneOrMany, ZeroOneOrMany, ByteSize, ByteSizeExt};

    #[test]
    fn test_collections_no_serde_basic() {
        let collection = ZeroOneOrMany::many(vec![1, 2, 3]);
        assert_eq!(collection.len(), 3);
        
        let one_collection = OneOrMany::one(42);
        assert_eq!(one_collection.len(), 1);
        
        let size = 1024u64.bytes();
        assert_eq!(size.as_bytes(), 1024);
    }
}

#[cfg(all(feature = "macros", not(feature = "hashbrown")))]
mod macros_no_hashbrown_tests {
    use cyrup_sugars::macros::collections::*;
    use cyrup_sugars::macros::closures::*;

    #[test]
    fn test_macros_no_hashbrown_basic() {
        let vec = vec_of![1, 2, 3];
        assert_eq!(vec, vec![1, 2, 3]);
        
        let x = 42;
        let closure = capture!(x, |y| x + y);
        assert_eq!(closure(8), 50);
    }
}

#[cfg(all(feature = "async", not(feature = "tokio")))]
mod async_no_tokio_tests {
    use cyrup_sugars::{AsyncTask, AsyncResult};

    #[test]
    fn test_async_no_tokio_basic() {
        let task = AsyncTask::from_value(42);
        // Note: Can't await without tokio runtime, but we can verify the type exists
        
        let result: AsyncResult<i32, String> = AsyncResult::Ok(42);
        match result {
            AsyncResult::Ok(val) => assert_eq!(val, 42),
            AsyncResult::Err(_) => panic!("Should be Ok"),
        }
    }
}