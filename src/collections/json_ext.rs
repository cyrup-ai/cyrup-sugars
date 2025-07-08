//! Extension traits for JSON object syntax support with hashbrown
//!
//! This module provides extension traits that enable the clean JSON object syntax
//! for collection types when the `hashbrown-json` feature is enabled.

#[cfg(feature = "hashbrown-json")]
use super::{one_or_many::{EmptyListError, OneOrMany}, zero_one_or_many::ZeroOneOrMany};

/// Extension trait for types that can be constructed from hashbrown HashMap syntax.
/// 
/// This trait enables the JSON object syntax `{"key" => "value"}` for collection types
/// when used with the hashbrown macros.
#[cfg(feature = "hashbrown-json")]
pub trait JsonObjectExt: Sized {
    /// The error type returned when construction fails.
    type Error;
    
    /// Creates an instance from a hashbrown HashMap.
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        Self: From<Vec<(K, V)>> + TryFrom<Vec<(K, V)>, Error = Self::Error>;
    
    /// Creates an instance from a closure that returns a hashbrown HashMap.
    /// 
    /// This method is designed to work seamlessly with the hashbrown macros.
    fn from_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        Self: From<Vec<(K, V)>> + TryFrom<Vec<(K, V)>, Error = Self::Error>,
    {
        Self::from_hashmap(f())
    }
}

/// Extension trait for Vec<T> to support JSON object syntax.
#[cfg(feature = "hashbrown-json")]
impl<T> JsonObjectExt for Vec<T> {
    type Error = std::convert::Infallible;
    
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        Self: From<Vec<(K, V)>> + TryFrom<Vec<(K, V)>, Error = Self::Error>,
    {
        Ok(map.into_iter().collect())
    }
}

/// Extension trait for Option<T> to support JSON object syntax.
#[cfg(feature = "hashbrown-json")]
impl<T> JsonObjectExt for Option<Vec<T>> {
    type Error = std::convert::Infallible;
    
    fn from_hashmap<K, V>(map: ::hashbrown::HashMap<K, V>) -> Result<Self, Self::Error>
    where
        Self: From<Vec<(K, V)>> + TryFrom<Vec<(K, V)>, Error = Self::Error>,
    {
        let items: Vec<(K, V)> = map.into_iter().collect();
        Ok(if items.is_empty() { None } else { Some(items) })
    }
}

/// Extension methods for creating collections from JSON object syntax.
#[cfg(feature = "hashbrown-json")]
pub trait CollectionJsonExt {
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    /// 
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "hashbrown-json")]
    /// # {
    /// use cyrup_sugars::collections::{CollectionJsonExt, ZeroOneOrMany};
    /// use cyrup_sugars::macros::hashbrown::hash_map_fn;
    /// 
    /// let collection: ZeroOneOrMany<(&str, &str)> = ZeroOneOrMany::json(hash_map_fn! {
    ///     "beta" => "true",
    ///     "version" => "2.1.0",
    /// });
    /// # }
    /// ```
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>;
}

/// Extension methods for creating collections that may fail from JSON object syntax.
#[cfg(feature = "hashbrown-json")]
pub trait TryCollectionJsonExt {
    /// The error type returned when construction fails.
    type Error;
    
    /// Creates a collection from a closure that returns a hashbrown HashMap.
    /// 
    /// # Example
    /// ```rust
    /// # #[cfg(feature = "hashbrown-json")]
    /// # {
    /// use cyrup_sugars::collections::{TryCollectionJsonExt, OneOrMany};
    /// use cyrup_sugars::macros::hashbrown::hash_map_fn;
    /// 
    /// let collection: OneOrMany<(&str, &str)> = OneOrMany::try_json(hash_map_fn! {
    ///     "beta" => "true",
    ///     "version" => "2.1.0",
    /// }).unwrap();
    /// # }
    /// ```
    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        Self: Sized;
}

#[cfg(feature = "hashbrown-json")]
impl CollectionJsonExt for ZeroOneOrMany<(String, String)> {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(String, String)> = map.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        ZeroOneOrMany::many(items)
    }
}

#[cfg(feature = "hashbrown-json")]
impl<K, V> CollectionJsonExt for ZeroOneOrMany<(K, V)> {
    fn json<K2, V2, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K2, V2>,
        K2: Into<K>,
        V2: Into<V>,
    {
        ZeroOneOrMany::from_json(|| {
            f().into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect::<::hashbrown::HashMap<K, V>>()
        })
    }
}

#[cfg(feature = "hashbrown-json")]
impl TryCollectionJsonExt for OneOrMany<(String, String)> {
    type Error = EmptyListError;
    
    fn try_json<K, V, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        let map = f();
        let items: Vec<(String, String)> = map.into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        OneOrMany::many(items)
    }
}

#[cfg(feature = "hashbrown-json")]
impl<K, V> TryCollectionJsonExt for OneOrMany<(K, V)> {
    type Error = EmptyListError;
    
    fn try_json<K2, V2, F>(f: F) -> Result<Self, Self::Error>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K2, V2>,
        K2: Into<K>,
        V2: Into<V>,
    {
        OneOrMany::from_json(|| {
            f().into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect::<::hashbrown::HashMap<K, V>>()
        })
    }
}

#[cfg(feature = "hashbrown-json")]
impl CollectionJsonExt for Vec<(String, String)> {
    fn json<K, V, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
        K: Into<String>,
        V: Into<String>,
    {
        f().into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    }
}

#[cfg(feature = "hashbrown-json")]
impl<K, V> CollectionJsonExt for Vec<(K, V)> {
    fn json<K2, V2, F>(f: F) -> Self
    where
        F: FnOnce() -> ::hashbrown::HashMap<K2, V2>,
        K2: Into<K>,
        V2: Into<V>,
    {
        f().into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect()
    }
}