//! # Cyrup Sugars
//!
//! Syntactic sugar utilities for Rust - collections, async patterns, and macros.
//!
//! This crate provides ergonomic utilities organized into feature-gated modules:
//!
//! ## Features
//!
//! - `collections` - Enhanced collection types like `ZeroOneOrMany`, `OneOrMany`, and `ByteSize`
//! - `async` - Async utilities with the "always unwrapped" pattern using `AsyncTask` and `AsyncStream`
//! - `macros` - Convenient macros for collections and async operations
//! - `hashbrown-json` - 🔥 Amazing hashbrown HashMap macros with full JSON object support
//! - `gix-interop` - Git object ID optimized hash tables
//!
//! ## Example
//!
//! ```rust
//! use cyrup_sugars::collections::ByteSizeExt;
//! use cyrup_sugars::{AsyncTask, AsyncResult};
//!
//! // Ergonomic byte sizes
//! let cache_size = 512.mb();
//! println!("Cache size: {} bytes", cache_size.as_bytes());
//!
//! // Type-safe async operations (no raw Results allowed)
//! let task = AsyncTask::from_value(42);
//! // let bad_task = AsyncTask::from_value(Ok(42)); // Compile error!
//! ```
//!
//! ### 🔥 Hashbrown JSON Syntax (with `hashbrown-json` feature)
//!
//! ```ignore
//! use cyrup_sugars::collections::{ZeroOneOrMany, OneOrMany};
//! use cyrup_sugars::macros::hashbrown::hash_map;
//! use serde_json;
//!
//! // Semantic JSON mapping with blazing fast hashbrown
//! let config = hash_map! {
//!     "servers" => ZeroOneOrMany::many(vec!["api.com", "db.com"]),
//!     "endpoints" => OneOrMany::one("primary.api.com")
//! };
//! let json = serde_json::to_string_pretty(&config)?;
//!
//! // Flexible deserialization - handles null, single values, or arrays
//! let from_null: ZeroOneOrMany<String> = serde_json::from_str("null")?;
//! let from_single: ZeroOneOrMany<String> = serde_json::from_str(r#""hello""#)?;
//! let from_array: ZeroOneOrMany<String> = serde_json::from_str(r#"["hello", "world"]"#)?;
//! ```

#![feature(auto_traits, negative_impls)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![forbid(unsafe_code)]

// Conditionally compile modules based on features
#[cfg(feature = "collections")]
pub mod collections;

#[cfg(feature = "async")]
pub mod r#async;

#[cfg(feature = "macros")]
pub mod macros;

#[cfg(feature = "gix-interop")]
pub mod external;

#[cfg(all(feature = "collections", feature = "hashbrown-json"))]
pub mod builders;

// Re-export commonly used types from collections
#[cfg(feature = "collections")]
pub use collections::{
    byte_size::{ByteSize, ByteSizeExt},
    one_or_many::OneOrMany,
    zero_one_or_many::ZeroOneOrMany,
};

// Re-export JSON extension traits when both features are enabled
#[cfg(all(feature = "collections", feature = "hashbrown-json"))]
pub use collections::{
    JsonObjectExtStringString, JsonObjectExtStringV, 
    JsonObjectExtKString, JsonObjectExtKV,
    CollectionJsonExtStringString, CollectionJsonExtStringV, 
    CollectionJsonExtKString, CollectionJsonExtKV,
    TryCollectionJsonExtStringString, TryCollectionJsonExtStringV, 
    TryCollectionJsonExtKString, TryCollectionJsonExtKV
};

// Re-export async utilities
#[cfg(feature = "async")]
pub use r#async::{
    future_ext::FutureExt, stream_ext::StreamExt, AsyncResult, AsyncResultChunk, AsyncStream,
    AsyncTask, NotResult,
};
