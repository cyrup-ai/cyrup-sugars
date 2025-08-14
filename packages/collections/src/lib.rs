//! Collection utilities and data structures

#![feature(auto_traits, negative_impls)]

pub mod byte_size;
/// A non-empty collection guaranteed to hold at least one value.
pub mod one_or_many;
/// A collection that can hold zero, one, or many values, optimized for minimal allocations.
pub mod zero_one_or_many;

/// Extension traits for array tuple syntax support
pub mod array_tuple_ext;

// Re-export main types
pub use byte_size::{ByteSize, ByteSizeExt};
pub use one_or_many::OneOrMany;
pub use zero_one_or_many::ZeroOneOrMany;

// Re-export extension traits
#[cfg(feature = "array-tuples")]
pub use array_tuple_ext::{
    ArrayTupleObjectExtKString, ArrayTupleObjectExtKV, ArrayTupleObjectExtStringString,
    ArrayTupleObjectExtStringV, CollectionArrayTupleExtKString, CollectionArrayTupleExtKV,
    CollectionArrayTupleExtStringString, CollectionArrayTupleExtStringV,
    TryCollectionArrayTupleExtKString, TryCollectionArrayTupleExtKV,
    TryCollectionArrayTupleExtStringString, TryCollectionArrayTupleExtStringV,
};

/// Creates a closure that returns a hashbrown HashMap from array tuple syntax
///
/// This macro enables the convenient `[("key", "value")]` syntax in builder patterns
///
/// Usage:
/// ```rust
/// use sugars_collections::hash_map;
///
/// let config = hash_map![("api_key", "secret"), ("timeout", "30s")];
/// let map = config(); // Returns hashbrown::HashMap<&str, &str>
/// ```
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! hash_map {
    [ $($key:expr, $value:expr),* $(,)? ] => {
        || {
            let mut map = ::hashbrown::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }
    };
}

/// Transforms array tuple syntax in builder chains to work with hash_map! macro
///
/// This macro makes `[("key", "value")]` syntax work transparently as closures
/// by automatically wrapping array tuples with the appropriate hash_map! calls.
///
/// Usage:
/// ```ignore
/// use sugars_collections::array_tuple_closure;
///
/// array_tuple_closure! {
///     FluentAi::agent_role("example")
///         .additional_params([("beta", "true")])
///         .metadata([("key", "val"), ("foo", "bar")])
///         .tools((Tool::<Perplexity>::new([("citations", "true")]),))
/// }
/// ```
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! array_tuple_closure {
    // Transform the entire expression tree
    ( $($input:tt)* ) => {
        array_tuple_closure_internal! { $($input)* }
    };
}

/// Internal implementation for array tuple closure transformation
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! array_tuple_closure_internal {
    // Base case: find and transform array tuple patterns
    ( $($tokens:tt)* ) => {
        array_tuple_closure_replace! { $($tokens)* }
    };
}

/// Recursively replaces array tuple patterns with hash_map! calls
/// Works at the token level to handle `[("key", "value")]` syntax before Rust parsing
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! array_tuple_closure_replace {
    // Empty case
    () => {};

    // Handle array tuple blocks first - highest priority
    ( $($prefix:tt)* [ $($inner:tt)+ ] $($suffix:tt)* ) => {
        array_tuple_closure_replace_inner! {
            prefix: [ $($prefix)* ]
            block: [ $($inner)+ ]
            suffix: [ $($suffix)* ]
        }
    };

    // No array tuple blocks found - pass through unchanged
    ( $($tokens:tt)* ) => {
        $($tokens)*
    };
}

/// Internal helper to process detected array tuple blocks
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! array_tuple_closure_replace_inner {
    // Check if block contains tuple patterns
    ( prefix: [ $($prefix:tt)* ] block: [ $($inner:tt)+ ] suffix: [ $($suffix:tt)* ] ) => {
        array_tuple_closure_check_tuples! {
            prefix: [ $($prefix)* ]
            inner: [ $($inner)+ ]
            suffix: [ $($suffix)* ]
        }
    };
}

/// Check for tuple patterns and transform if found
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! array_tuple_closure_check_tuples {
    // Any tuple pattern - transform to hash_map_fn! call
    ( prefix: [ $($prefix:tt)* ] inner: [ $($inner:tt)+ ] suffix: [ $($suffix:tt)* ] ) => {
        array_tuple_closure_tuple_check! {
            prefix: [ $($prefix)* ]
            inner: [ $($inner)+ ]
            suffix: [ $($suffix)* ]
        }
    };
}

/// Check if inner tokens contain tuples and transform appropriately
#[cfg(feature = "array-tuples")]
#[macro_export]
macro_rules! array_tuple_closure_tuple_check {
    // Check for tuple pattern in tokens
    ( prefix: [ $($prefix:tt)* ] inner: [ $($pre:tt)* ( $($tuple:tt)* ) $($post:tt)* ] suffix: [ $($suffix:tt)* ] ) => {
        array_tuple_closure_replace! { $($prefix)* sugars_macros::hash_map_fn! [ $($pre)* ( $($tuple)* ) $($post)* ] $($suffix)* }
    };

    // No tuple found - keep original block
    ( prefix: [ $($prefix:tt)* ] inner: [ $($inner:tt)+ ] suffix: [ $($suffix:tt)* ] ) => {
        array_tuple_closure_replace! { $($prefix)* [ $($inner)+ ] $($suffix)* }
    };
}
