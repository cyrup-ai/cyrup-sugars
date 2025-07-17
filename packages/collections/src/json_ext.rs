//! Extension traits for JSON object syntax support with hashbrown
//!
//! This module provides extension traits that enable the clean JSON object syntax
//! for collection types when the `hashbrown-json` feature is enabled.

#[cfg(feature = "hashbrown-json")]
use super::{
    one_or_many::{EmptyListError, OneOrMany},
    zero_one_or_many::ZeroOneOrMany,
};

/// Marker trait to exclude String types from generic implementations
#[cfg(feature = "hashbrown-json")]
pub auto trait NotString {}

#[cfg(feature = "hashbrown-json")]
impl !NotString for String {}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - String,String case.
#[cfg(feature = "hashbrown-json")]
pub trait JsonObjectExtStringString: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<String>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - String,V case.
#[cfg(feature = "hashbrown-json")]
pub trait JsonObjectExtStringV<V1: NotString>: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<String>,
        V: Into<V1>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - K,String case.
#[cfg(feature = "hashbrown-json")]
pub trait JsonObjectExtKString<K1: NotString>: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<String>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for types that can be constructed from hashbrown HashMap syntax - K,V case.
#[cfg(feature = "hashbrown-json")]
pub trait JsonObjectExtKV<K1: NotString, V1: NotString>: Sized {
    /// The error type returned when construction fails.
    type Error;

    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        K: Into<K1>,
        V: Into<V1>;

    /// Creates an instance from a closure that returns a hashbrown HashMap.
    fn from_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for Vec<(String, String)> to support JSON object syntax.
#[cfg(feature = "hashbrown-json")]
impl JsonObjectExtStringString for Vec<(String, String)> {
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
#[cfg(feature = "hashbrown-json")]
impl<V1: NotString> JsonObjectExtStringV<V1> for Vec<(String, V1)> {
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
#[cfg(feature = "hashbrown-json")]
impl<K1: NotString> JsonObjectExtKString<K1> for Vec<(K1, String)> {
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
#[cfg(feature = "hashbrown-json")]
impl<K1: NotString, V1: NotString> JsonObjectExtKV<K1, V1> for Vec<(K1, V1)> {
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
#[cfg(feature = "hashbrown-json")]
impl JsonObjectExtStringString for Option<Vec<(String, String)>> {
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
#[cfg(feature = "hashbrown-json")]
impl<V1: NotString> JsonObjectExtStringV<V1> for Option<Vec<(String, V1)>> {
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
#[cfg(feature = "hashbrown-json")]
impl<K1: NotString> JsonObjectExtKString<K1> for Option<Vec<(K1, String)>> {
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
#[cfg(feature = "hashbrown-json")]
impl<K1: NotString, V1: NotString> JsonObjectExtKV<K1, V1> for Option<Vec<(K1, V1)>> {
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
#[cfg(feature = "hashbrown-json")]
pub trait CollectionJsonExtStringString {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>;
}

/// Extension methods for creating collections from JSON object syntax - String,V case.
#[cfg(feature = "hashbrown-json")]
pub trait CollectionJsonExtStringV<V1: NotString> {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<V1>;
}

/// Extension methods for creating collections from JSON object syntax - K,String case.
#[cfg(feature = "hashbrown-json")]
pub trait CollectionJsonExtKString<K1: NotString> {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<String>;
}

/// Extension methods for creating collections from JSON object syntax - K,V case.
#[cfg(feature = "hashbrown-json")]
pub trait CollectionJsonExtKV<K1: NotString, V1: NotString> {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<K1>,
        V: Into<V1>;
}

/// Extension methods for creating collections that may fail from JSON object syntax - String,String case.
#[cfg(feature = "hashbrown-json")]
pub trait TryCollectionJsonExtStringString {
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
#[cfg(feature = "hashbrown-json")]
pub trait TryCollectionJsonExtStringV<V1: NotString> {
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
#[cfg(feature = "hashbrown-json")]
pub trait TryCollectionJsonExtKString<K1: NotString> {
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
#[cfg(feature = "hashbrown-json")]
pub trait TryCollectionJsonExtKV<K1: NotString, V1: NotString> {
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

#[cfg(feature = "hashbrown-json")]
impl TryCollectionJsonExtStringString for OneOrMany<(String, String)> {
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

#[cfg(feature = "hashbrown-json")]
impl<V1> TryCollectionJsonExtStringV<V1> for OneOrMany<(String, V1)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1> TryCollectionJsonExtKString<K1> for OneOrMany<(K1, String)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1, V1> TryCollectionJsonExtKV<K1, V1> for OneOrMany<(K1, V1)>
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

#[cfg(feature = "hashbrown-json")]
impl CollectionJsonExtStringString for Vec<(String, String)> {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        f().into_iter().map(|(k, v)| (k.into(), v.into())).collect()
    }
}

#[cfg(feature = "hashbrown-json")]
impl<V1> CollectionJsonExtStringV<V1> for Vec<(String, V1)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1> CollectionJsonExtKString<K1> for Vec<(K1, String)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1, V1> CollectionJsonExtKV<K1, V1> for Vec<(K1, V1)>
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
#[cfg(feature = "hashbrown-json")]
impl CollectionJsonExtStringString for ZeroOneOrMany<(String, String)> {
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

#[cfg(feature = "hashbrown-json")]
impl<V1> CollectionJsonExtStringV<V1> for ZeroOneOrMany<(String, V1)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1> CollectionJsonExtKString<K1> for ZeroOneOrMany<(K1, String)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1, V1> CollectionJsonExtKV<K1, V1> for ZeroOneOrMany<(K1, V1)>
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

// ZeroOneOrMany TryCollectionJsonExt implementations
#[cfg(feature = "hashbrown-json")]
impl TryCollectionJsonExtStringString for ZeroOneOrMany<(String, String)> {
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

#[cfg(feature = "hashbrown-json")]
impl<V1> TryCollectionJsonExtStringV<V1> for ZeroOneOrMany<(String, V1)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1> TryCollectionJsonExtKString<K1> for ZeroOneOrMany<(K1, String)>
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

#[cfg(feature = "hashbrown-json")]
impl<K1, V1> TryCollectionJsonExtKV<K1, V1> for ZeroOneOrMany<(K1, V1)>
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
