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
//! - `array-tuples` - 🔥 Amazing hashbrown HashMap macros with array tuple syntax support
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
//! ### 🔥 Hashbrown Array Tuple Syntax (with `array-tuples` feature)
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

/// Closure macros for elegant stream processing with zero-allocation pattern matching
pub mod closures;

// Re-export modules from workspace packages
pub use sugars_collections as collections;

/// Async utilities with the "always unwrapped" pattern.
///
/// This module provides `AsyncTask` and `AsyncStream` types that enforce
/// proper error handling at the type level, preventing raw `Result` types
/// from being wrapped in async primitives.
pub mod r#async {
    pub use sugars_async_stream::*;
    pub use sugars_async_task::*;
}

pub use sugars_macros as macros;

pub use sugars_gix as external;

pub use sugars_builders as builders;

// Re-export commonly used types from collections
pub use sugars_collections::{ByteSize, ByteSizeExt, OneOrMany, ZeroOneOrMany};

// Re-export array tuple extension traits when both features are enabled
#[cfg(feature = "array-tuples")]
pub use sugars_collections::{
    ArrayTupleObjectExtKString, ArrayTupleObjectExtKV, ArrayTupleObjectExtStringString,
    ArrayTupleObjectExtStringV, CollectionArrayTupleExtKString, CollectionArrayTupleExtKV,
    CollectionArrayTupleExtStringString, CollectionArrayTupleExtStringV,
    TryCollectionArrayTupleExtKString, TryCollectionArrayTupleExtKV,
    TryCollectionArrayTupleExtStringString, TryCollectionArrayTupleExtStringV,
};

// Re-export async utilities
pub use r#async::{
    AsyncResult, AsyncResultChunk, AsyncStream, AsyncTask, FutureExt, NotResult, StreamExt,
};

// Re-export JSON syntax macros for array-tuples feature
#[cfg(feature = "array-tuples")]
pub use sugars_collections::hash_map;
#[cfg(feature = "array-tuples")]
pub use sugars_macros::hash_map_fn;

/// Prelude module that brings common macros and types into scope
pub mod prelude {

    // Re-export commonly used types
    pub use crate::{
        AsyncResult, AsyncStream, AsyncTask, ByteSize, ByteSizeExt, OneOrMany, ZeroOneOrMany,
    };

    // Re-export JSON syntax macros when array-tuples feature is enabled
    #[cfg(feature = "array-tuples")]
    pub use crate::{hash_map, hash_map_fn};

    // Re-export async utilities
    pub use crate::r#async::{FutureExt, NotResult, StreamExt};

    // Re-export builder utilities
    pub use crate::builders::{ChunkHandler, MessageChunk};

    // Re-export macros for elegant stream processing (from local closures module)
    pub use crate::on_result;
}
