//! Extension traits for array tuple syntax support with hashbrown
//!
//! This module provides extension traits that enable the clean array tuple syntax
//! for collection types when the `array-tuples` feature is enabled.

#[cfg(feature = "array-tuples")]
use super::{
    one_or_many::{EmptyListError, OneOrMany},
    zero_one_or_many::ZeroOneOrMany,
};

/// Marker trait to exclude String types from generic implementations
#[cfg(feature = "array-tuples")]
pub auto trait NotString {}

#[cfg(feature = "array-tuples")]
impl !NotString for String {}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - String,String case.
#[cfg(feature = "array-tuples")]
pub trait ArrayTupleObjectExtStringString: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<String>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_array_tuple<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - String,V case.
#[cfg(feature = "array-tuples")]
pub trait ArrayTupleObjectExtStringV<V1: NotString>: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<V1>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_array_tuple<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - K,String case.
#[cfg(feature = "array-tuples")]
pub trait ArrayTupleObjectExtKString<K1: NotString>: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<String>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_array_tuple<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - K,V case.
#[cfg(feature = "array-tuples")]
pub trait ArrayTupleObjectExtKV<K1: NotString, V1: NotString>: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<V1>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_array_tuple<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for Vec<(String, String)> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl ArrayTupleObjectExtStringString for Vec<(String, String)> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<String>,
    {
        Ok(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

/// Extension trait for Vec<(String, V1)> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl<V1: NotString> ArrayTupleObjectExtStringV<V1> for Vec<(String, V1)> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<V1>,
    {
        Ok(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

/// Extension trait for Vec<(K1, String)> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl<K1: NotString> ArrayTupleObjectExtKString<K1> for Vec<(K1, String)> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<String>,
    {
        Ok(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

/// Extension trait for Vec<(K1, V1)> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl<K1: NotString, V1: NotString> ArrayTupleObjectExtKV<K1, V1> for Vec<(K1, V1)> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<V1>,
    {
        Ok(map.into_iter().map(|(k, v)| (k.into(), v.into())).collect())
    }
}

/// Extension trait for Option<Vec<(String, String)>> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl ArrayTupleObjectExtStringString for Option<Vec<(String, String)>> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<String>,
    {
        let items: Vec<(String, String)> =
            map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(if items.is_empty() { None } else { Some(items) })
    }
}

/// Extension trait for Option<Vec<(String, V1)>> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl<V1: NotString> ArrayTupleObjectExtStringV<V1> for Option<Vec<(String, V1)>> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<V1>,
    {
        let items: Vec<(String, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(if items.is_empty() { None } else { Some(items) })
    }
}

/// Extension trait for Option<Vec<(K1, String)>> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl<K1: NotString> ArrayTupleObjectExtKString<K1> for Option<Vec<(K1, String)>> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<String>,
    {
        let items: Vec<(K1, String)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(if items.is_empty() { None } else { Some(items) })
    }
}

/// Extension trait for Option<Vec<(K1, V1)>> to support JSON object syntax.
#[cfg(feature = "array-tuples")]
impl<K1: NotString, V1: NotString> ArrayTupleObjectExtKV<K1, V1> for Option<Vec<(K1, V1)>> {
    type Error = std::convert::Infallible;

    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<V1>,
    {
        let items: Vec<(K1, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(if items.is_empty() { None } else { Some(items) })
    }
}

/// Extension methods for creating collections from JSON object syntax - String,String case.
#[cfg(feature = "array-tuples")]
pub trait CollectionArrayTupleExtStringString {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>;
}

/// Extension methods for creating collections from JSON object syntax - String,V case.
#[cfg(feature = "array-tuples")]
pub trait CollectionArrayTupleExtStringV<V1: NotString> {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>;
}

/// Extension methods for creating collections from JSON object syntax - K,String case.
#[cfg(feature = "array-tuples")]
pub trait CollectionArrayTupleExtKString<K1: NotString> {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>;
}

/// Extension methods for creating collections from JSON object syntax - K,V case.
#[cfg(feature = "array-tuples")]
pub trait CollectionArrayTupleExtKV<K1: NotString, V1: NotString> {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>;
}

/// Extension methods for creating collections that may fail from JSON object syntax - String,String case.
#[cfg(feature = "array-tuples")]
pub trait TryCollectionArrayTupleExtStringString {
    /// The error type returned when construction fails.
    type Error;

    /// Tries to create a collection from a closure that returns a hashbrown HashMap.
    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
        Self: Sized;
}

/// Extension methods for creating collections that may fail from JSON object syntax - String,V case.
#[cfg(feature = "array-tuples")]
pub trait TryCollectionArrayTupleExtStringV<V1: NotString> {
    /// The error type returned when construction fails.
    type Error;

    /// Tries to create a collection from a closure that returns a hashbrown HashMap.
    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
        Self: Sized;
}

/// Extension methods for creating collections that may fail from JSON object syntax - K,String case.
#[cfg(feature = "array-tuples")]
pub trait TryCollectionArrayTupleExtKString<K1: NotString> {
    /// The error type returned when construction fails.
    type Error;

    /// Tries to create a collection from a closure that returns a hashbrown HashMap.
    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
        Self: Sized;
}

/// Extension methods for creating collections that may fail from JSON object syntax - K,V case.
#[cfg(feature = "array-tuples")]
pub trait TryCollectionArrayTupleExtKV<K1: NotString, V1: NotString> {
    /// The error type returned when construction fails.
    type Error;

    /// Tries to create a collection from a closure that returns a hashbrown HashMap.
    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
        Self: Sized;
}

#[cfg(feature = "array-tuples")]
impl TryCollectionArrayTupleExtStringString for OneOrMany<(String, String)> {
    type Error = EmptyListError;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(String, String)> =
            map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        OneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl<V1> TryCollectionArrayTupleExtStringV<V1> for OneOrMany<(String, V1)>
where
    V1: NotString,
{
    type Error = EmptyListError;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
    {
        let map = f();
        let items: Vec<(String, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        OneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl<K1> TryCollectionArrayTupleExtKString<K1> for OneOrMany<(K1, String)>
where
    K1: NotString,
{
    type Error = EmptyListError;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(K1, String)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        OneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl<K1, V1> TryCollectionArrayTupleExtKV<K1, V1> for OneOrMany<(K1, V1)>
where
    K1: NotString,
    V1: NotString,
{
    type Error = EmptyListError;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
    {
        let map = f();
        let items: Vec<(K1, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        OneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl CollectionArrayTupleExtStringString for Vec<(String, String)> {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        f().into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}

#[cfg(feature = "array-tuples")]
impl<V1> CollectionArrayTupleExtStringV<V1> for Vec<(String, V1)>
where
    V1: NotString,
{
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
    {
        f().into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}

#[cfg(feature = "array-tuples")]
impl<K1> CollectionArrayTupleExtKString<K1> for Vec<(K1, String)>
where
    K1: NotString,
{
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
    {
        f().into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}

#[cfg(feature = "array-tuples")]
impl<K1, V1> CollectionArrayTupleExtKV<K1, V1> for Vec<(K1, V1)>
where
    K1: NotString,
    V1: NotString,
{
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
    {
        f().into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}

// ZeroOneOrMany implementations
#[cfg(feature = "array-tuples")]
impl CollectionArrayTupleExtStringString for ZeroOneOrMany<(String, String)> {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(String, String)> =
            map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        ZeroOneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl<V1> CollectionArrayTupleExtStringV<V1> for ZeroOneOrMany<(String, V1)>
where
    V1: NotString,
{
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
    {
        let map = f();
        let items: Vec<(String, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        ZeroOneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl<K1> CollectionArrayTupleExtKString<K1> for ZeroOneOrMany<(K1, String)>
where
    K1: NotString,
{
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(K1, String)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        ZeroOneOrMany::many(items)
    }
}

#[cfg(feature = "array-tuples")]
impl<K1, V1> CollectionArrayTupleExtKV<K1, V1> for ZeroOneOrMany<(K1, V1)>
where
    K1: NotString,
    V1: NotString,
{
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
    {
        let map = f();
        let items: Vec<(K1, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        ZeroOneOrMany::many(items)
    }
}

// ZeroOneOrMany TryCollectionArrayTupleExt implementations
#[cfg(feature = "array-tuples")]
impl TryCollectionArrayTupleExtStringString for ZeroOneOrMany<(String, String)> {
    type Error = std::convert::Infallible;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(String, String)> =
            map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(ZeroOneOrMany::many(items))
    }
}

#[cfg(feature = "array-tuples")]
impl<V1> TryCollectionArrayTupleExtStringV<V1> for ZeroOneOrMany<(String, V1)>
where
    V1: NotString,
{
    type Error = std::convert::Infallible;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
    {
        let map = f();
        let items: Vec<(String, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(ZeroOneOrMany::many(items))
    }
}

#[cfg(feature = "array-tuples")]
impl<K1> TryCollectionArrayTupleExtKString<K1> for ZeroOneOrMany<(K1, String)>
where
    K1: NotString,
{
    type Error = std::convert::Infallible;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(K1, String)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(ZeroOneOrMany::many(items))
    }
}

#[cfg(feature = "array-tuples")]
impl<K1, V1> TryCollectionArrayTupleExtKV<K1, V1> for ZeroOneOrMany<(K1, V1)>
where
    K1: NotString,
    V1: NotString,
{
    type Error = std::convert::Infallible;

    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
    {
        let map = f();
        let items: Vec<(K1, V1)> = map.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        Ok(ZeroOneOrMany::many(items))
    }
}
