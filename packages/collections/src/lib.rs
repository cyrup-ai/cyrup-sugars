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
        || {
            let mut map = ::hashbrown::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

/// Transforms JSON-like syntax in builder chains to work with hash_map! macro
///
/// This macro makes `{"key" => "value"}` syntax work transparently as closures
/// by automatically wrapping JSON objects with the appropriate hash_map! calls.
///
/// Usage:
/// ```rust
/// use sugars_collections::json_closure;
///
/// json_closure! {
///     FluentAi::agent_role("example")
///         .additional_params({"beta" => "true"})
///         .metadata({"key" => "val", "foo" => "bar"})
///         .tools((Tool::<Perplexity>::new({"citations" => "true"}),))
/// }
/// ```
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure {
    // Transform the entire expression tree
    ( $($input:tt)* ) => {
        json_closure_internal! { $($input)* }
    };
}

/// Internal implementation for JSON closure transformation
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_internal {
    // Base case: find and transform JSON patterns
    ( $($tokens:tt)* ) => {
        json_closure_replace! { $($tokens)* }
    };
}

/// Recursively replaces JSON patterns with hash_map! calls
/// Works at the token level to handle `{"key" => "value"}` syntax before Rust parsing
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_replace {
    // Empty case
    () => {};
    
    // Handle JSON object blocks first - highest priority
    ( $($prefix:tt)* { $($inner:tt)+ } $($suffix:tt)* ) => {
        json_closure_replace_inner! { 
            prefix: [ $($prefix)* ]
            block: { $($inner)+ }
            suffix: [ $($suffix)* ]
        }
    };
    
    // No JSON blocks found - pass through unchanged
    ( $($tokens:tt)* ) => {
        $($tokens)*
    };
}

/// Internal helper to process detected JSON blocks
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_replace_inner {
    // Check if block contains arrow patterns
    ( prefix: [ $($prefix:tt)* ] block: { $($inner:tt)+ } suffix: [ $($suffix:tt)* ] ) => {
        json_closure_check_arrows! {
            prefix: [ $($prefix)* ]
            inner: [ $($inner)+ ]
            suffix: [ $($suffix)* ]
        }
    };
}

/// Check for arrow patterns and transform if found
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_check_arrows {
    // Any arrow pattern - transform to hash_map_fn! call
    ( prefix: [ $($prefix:tt)* ] inner: [ $($inner:tt)+ ] suffix: [ $($suffix:tt)* ] ) => {
        json_closure_arrow_check! {
            prefix: [ $($prefix)* ]
            inner: [ $($inner)+ ]
            suffix: [ $($suffix)* ]
        }
    };
}

/// Check if inner tokens contain arrows and transform appropriately
#[cfg(feature = "hashbrown-json")]
#[macro_export]
macro_rules! json_closure_arrow_check {
    // Check for => pattern in tokens
    ( prefix: [ $($prefix:tt)* ] inner: [ $($pre:tt)* => $($post:tt)* ] suffix: [ $($suffix:tt)* ] ) => {
        json_closure_replace! { $($prefix)* sugars_macros::hash_map_fn! { $($pre)* => $($post)* } $($suffix)* }
    };
    
    // No arrow found - keep original block
    ( prefix: [ $($prefix:tt)* ] inner: [ $($inner:tt)+ ] suffix: [ $($suffix:tt)* ] ) => {
        json_closure_replace! { $($prefix)* { $($inner)+ } $($suffix)* }
    };
}

