//! Collection utilities and data structures

#![feature(auto_traits, negative_impls)]

pub mod byte_size;
/// A non-empty collection guaranteed to hold at least one value.
pub mod one_or_many;
/// A collection that can hold zero, one, or many values, optimized for minimal allocations.
pub mod zero_one_or_many;

/// Extension traits for JSON object syntax support
pub mod json_ext;

// Re-export main types
pub use byte_size::{ByteSize, ByteSizeExt};
pub use one_or_many::OneOrMany;
pub use zero_one_or_many::ZeroOneOrMany;

// Re-export extension traits
#[cfg(feature = "hashbrown-json")]
pub use json_ext::{
    CollectionJsonExtKString, CollectionJsonExtKV, CollectionJsonExtStringString,
    CollectionJsonExtStringV, JsonObjectExtKString, JsonObjectExtKV, JsonObjectExtStringString,
    JsonObjectExtStringV, TryCollectionJsonExtKString, TryCollectionJsonExtKV,
    TryCollectionJsonExtStringString, TryCollectionJsonExtStringV,
};

/// Creates a closure that returns a hashbrown HashMap from JSON-like syntax
///
/// This macro enables the beautiful `{"key" => "value"}` syntax in builder patterns
///
/// Usage:
/// ```rust
/// use sugars_collections::hash_map;
///
/// let config = hash_map!{"api_key" => "secret", "timeout" => "30s"};
/// let map = config(); // Returns hashbrown::HashMap<&str, &str>
/// ```
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! hash_map {
    { $($key:expr => $value:expr),* $(,)? } => {
        ||
        {
            let mut map = ::hashbrown::HashMap::new();
            $(                map.insert($key, $value);
            )*
            map
        }
    };
}

/// Creates a closure that handles Result values with separate expressions for Ok and Err cases.
#[macro_export]
macro_rules! on_result {
    (Ok => $ok:expr, Err => $err:expr) => {
        move |__res| match __res {
            Ok(chunk) => Ok($ok),
            Err(err) => Err($err),
        }
    };
}

/// Creates a closure that processes stream chunks with the provided expression.
#[macro_export]
macro_rules! on_chunk {
    ($expr:expr) => {
        move |__chunk| $expr
    };
}

/// Creates a closure that handles errors with the provided expression.
#[macro_export]
macro_rules! on_error {
    ($expr:expr) => {
        move |__err| $expr
    };
}

/// Creates an async closure that processes results with the provided pattern and body.
#[macro_export]
macro_rules! await_result {
    ($param:pat => $body:expr) => {
        move |$param| async move { $body }
    };
}

/// Creates an async closure that processes successful values with the provided pattern and body.
#[macro_export]
macro_rules! await_ok {
    ($param:pat => $body:expr) => {
        move |$param| async move {
            $body;
        }
    };
}

/// JSON closure macro that transforms JSON-like syntax to hash_map! calls
/// 
/// Transforms `{"key" => "value"}` syntax to proper HashMap construction
/// Used by the json_syntax proc macro attribute
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure {
    ( $($input:tt)* ) => {
        json_closure_internal! { $($input)* }
    };
}

/// Internal implementation for JSON closure transformation
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_internal {
    // Transform tokens by checking for arrow syntax and replacing JSON patterns
    ( $($tokens:tt)* ) => {
        json_closure_arrow_check! { $($tokens)* }
    };
}

/// Check for arrow syntax in blocks
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_arrow_check {
    // Check for arrow syntax in blocks
    ({ $($tokens:tt)* }) => {
        json_closure_replace! { { $($tokens)* } }
    };
    // Pass through other tokens
    ($tokens:tt) => {
        $tokens
    };
}

/// Replace JSON patterns with hash_map! calls
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_replace {
    // Transform JSON object with arrow syntax to hash_map! call
    ({ $($key:literal => $value:expr),* $(,)? }) => {
        $crate::hash_map! { $($key => $value),* }
    };
    // Transform JSON object with mixed syntax
    ({ $($key:expr => $value:expr),* $(,)? }) => {
        $crate::hash_map! { $($key => $value),* }
    };
    // Pass through blocks that don't match JSON syntax
    ({ $($tokens:tt)* }) => {
        { $($tokens)* }
    };
    // Pass through other tokens unchanged
    ($tokens:tt) => {
        $tokens
    };
}
