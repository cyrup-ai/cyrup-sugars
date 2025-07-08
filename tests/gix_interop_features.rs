//! Tests for gix-interop features

#[cfg(feature = "gix-interop")]
mod gix_interop_tests {
    use cyrup_sugars::external::gix_hashtable::*;
    use gix_hash::{ObjectId, Kind};
    use std::collections::HashMap;

    #[test]
    fn test_gix_interop_object_id_map() {
        let mut map = ObjectIdMap::new();
        
        // Create a test ObjectId
        let oid = ObjectId::empty_tree(Kind::Sha1);
        
        map.insert(oid, "test_value");
        assert_eq!(map.get(&oid), Some(&"test_value"));
        assert_eq!(map.len(), 1);
        
        // Test overwrite
        map.insert(oid, "new_value");
        assert_eq!(map.get(&oid), Some(&"new_value"));
        assert_eq!(map.len(), 1);
        
        // Test removal
        let removed = map.remove(&oid);
        assert_eq!(removed, Some("new_value"));
        assert_eq!(map.len(), 0);
        assert_eq!(map.get(&oid), None);
    }

    #[test]
    fn test_gix_interop_object_id_set() {
        let mut set = ObjectIdSet::new();
        
        let oid1 = ObjectId::empty_tree(Kind::Sha1);
        let oid2 = ObjectId::empty_blob(Kind::Sha1);
        
        assert!(set.insert(oid1));
        assert!(set.insert(oid2));
        assert!(!set.insert(oid1)); // Already exists
        
        assert_eq!(set.len(), 2);
        assert!(set.contains(&oid1));
        assert!(set.contains(&oid2));
        
        assert!(set.remove(&oid1));
        assert!(!set.remove(&oid1)); // Already removed
        assert_eq!(set.len(), 1);
        assert!(!set.contains(&oid1));
        assert!(set.contains(&oid2));
    }

    #[test]
    fn test_gix_interop_object_id_map_iteration() {
        let mut map = ObjectIdMap::new();
        
        let oid1 = ObjectId::empty_tree(Kind::Sha1);
        let oid2 = ObjectId::empty_blob(Kind::Sha1);
        
        map.insert(oid1, "tree");
        map.insert(oid2, "blob");
        
        let mut collected: Vec<_> = map.iter().collect();
        collected.sort_by_key(|(_, &value)| value);
        
        assert_eq!(collected.len(), 2);
        // Values should be sorted as "blob", "tree"
        assert_eq!(collected[0].1, &"blob");
        assert_eq!(collected[1].1, &"tree");
    }

    #[test]
    fn test_gix_interop_object_id_set_iteration() {
        let mut set = ObjectIdSet::new();
        
        let oid1 = ObjectId::empty_tree(Kind::Sha1);
        let oid2 = ObjectId::empty_blob(Kind::Sha1);
        
        set.insert(oid1);
        set.insert(oid2);
        
        let collected: Vec<_> = set.iter().copied().collect();
        assert_eq!(collected.len(), 2);
        assert!(collected.contains(&oid1));
        assert!(collected.contains(&oid2));
    }

    #[test]
    fn test_gix_interop_with_capacity() {
        let map: ObjectIdMap<String> = ObjectIdMap::with_capacity(100);
        assert_eq!(map.len(), 0);
        
        let set = ObjectIdSet::with_capacity(50);
        assert_eq!(set.len(), 0);
    }

    #[test]
    fn test_gix_interop_clear() {
        let mut map = ObjectIdMap::new();
        let mut set = ObjectIdSet::new();
        
        let oid = ObjectId::empty_tree(Kind::Sha1);
        
        map.insert(oid, "value");
        set.insert(oid);
        
        assert_eq!(map.len(), 1);
        assert_eq!(set.len(), 1);
        
        map.clear();
        set.clear();
        
        assert_eq!(map.len(), 0);
        assert_eq!(set.len(), 0);
        assert!(map.is_empty());
        assert!(set.is_empty());
    }

    #[test]
    fn test_gix_interop_different_object_kinds() {
        let mut map = ObjectIdMap::new();
        
        let tree_oid = ObjectId::empty_tree(Kind::Sha1);
        let blob_oid = ObjectId::empty_blob(Kind::Sha1);
        let sha256_tree = ObjectId::empty_tree(Kind::Sha256);
        
        map.insert(tree_oid, "sha1_tree");
        map.insert(blob_oid, "sha1_blob");
        map.insert(sha256_tree, "sha256_tree");
        
        assert_eq!(map.len(), 3);
        assert_eq!(map.get(&tree_oid), Some(&"sha1_tree"));
        assert_eq!(map.get(&blob_oid), Some(&"sha1_blob"));
        assert_eq!(map.get(&sha256_tree), Some(&"sha256_tree"));
    }
}

#[cfg(all(feature = "gix-interop", feature = "collections"))]
mod gix_interop_collections_integration_tests {
    use cyrup_sugars::external::gix_hashtable::*;
    use cyrup_sugars::{OneOrMany, ZeroOneOrMany};
    use gix_hash::{ObjectId, Kind};

    #[test]
    fn test_gix_interop_collections_with_object_ids() {
        let oid1 = ObjectId::empty_tree(Kind::Sha1);
        let oid2 = ObjectId::empty_blob(Kind::Sha1);
        let oid3 = ObjectId::empty_tree(Kind::Sha256);
        
        let collection = ZeroOneOrMany::many(vec![oid1, oid2, oid3]);
        assert_eq!(collection.len(), 3);
        
        let one_collection = OneOrMany::many(vec![oid1, oid2]).unwrap();
        assert_eq!(one_collection.len(), 2);
    }

    #[test]
    fn test_gix_interop_collections_map_with_collections() {
        let mut map = ObjectIdMap::new();
        
        let oid1 = ObjectId::empty_tree(Kind::Sha1);
        let oid2 = ObjectId::empty_blob(Kind::Sha1);
        
        let collection1 = OneOrMany::many(vec!["file1.txt", "file2.txt"]).unwrap();
        let collection2 = ZeroOneOrMany::many(vec!["config.yml"]);
        
        map.insert(oid1, collection1);
        map.insert(oid2, collection2);
        
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&oid1).unwrap().len(), 2);
        assert_eq!(map.get(&oid2).unwrap().len(), 1);
    }
}