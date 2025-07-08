//! Collection utilities and data structures

pub mod byte_size;
/// A non-empty collection guaranteed to hold at least one value.
pub mod one_or_many;
/// A collection that can hold zero, one, or many values, optimized for minimal allocations.
pub mod zero_one_or_many;

#[cfg(feature = "hashbrown-json")]
/// Extension traits for JSON object syntax support
pub mod json_ext;

// Re-export extension traits
#[cfg(feature = "hashbrown-json")]
pub use json_ext::{
    JsonObjectExtStringString, JsonObjectExtStringV, 
    JsonObjectExtKString, JsonObjectExtKV,
    CollectionJsonExtStringString, CollectionJsonExtStringV, 
    CollectionJsonExtKString, CollectionJsonExtKV,
    TryCollectionJsonExtStringString, TryCollectionJsonExtStringV, 
    TryCollectionJsonExtKString, TryCollectionJsonExtKV
};
