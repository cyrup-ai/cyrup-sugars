// -----------------------------------------------------------------------------
// src/one_or_many.rs
// -----------------------------------------------------------------------------

use super::zero_one_or_many::ZeroOneOrMany;
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::FromIterator;
use std::marker::PhantomData;

/// A non-empty collection that holds one or many values of type `T`.
///
/// This struct wraps `ZeroOneOrMany<T>`, ensuring the `None` variant is never used.
/// It guarantees at least one element, with attempts to create an empty collection
/// resulting in an `EmptyListError`.
///
/// ### Immutability
/// All operations are immutable, returning new instances to preserve the non-empty
/// invariant and ensure thread-safety.
///
/// ### Serialization and Deserialization
/// Serializes to a JSON array with at least one element. Deserialization fails on
/// `null` or empty arrays, enforcing the non-empty constraint.
///
/// ### Performance
/// - **Zero Allocation**: Reuses `ZeroOneOrMany<T>`'s allocation strategy (`None` and
///   `One` avoid heap; `Many` pre-allocates).
/// - **Inlined Methods**: Critical methods are marked `#[inline]` for performance.
/// - **Minimal Cloning**: Most operations do not require `T: Clone`.
///
/// ### Examples
/// ```rust
/// let single = OneOrMany::one(42);
/// let multiple = OneOrMany::many(vec![1, 2, 3])?;
/// let pushed = single.with_pushed(43);
/// ```
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct OneOrMany<T>(ZeroOneOrMany<T>);

/// Error returned when attempting to create a `OneOrMany` from an empty collection.
#[derive(Debug)]
pub struct EmptyListError;

impl fmt::Display for EmptyListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OneOrMany cannot be empty")
    }
}

// Core API
impl<T> OneOrMany<T> {
    /// Creates a collection with a single element.
    #[inline]
    pub fn one(item: T) -> Self {
        OneOrMany(ZeroOneOrMany::One(item))
    }

    /// Creates a collection from a `Vec<T>`, failing if empty.
    #[inline]
    pub fn many(items: Vec<T>) -> Result<Self, EmptyListError> {
        if items.is_empty() {
            Err(EmptyListError)
        } else {
            Ok(OneOrMany(ZeroOneOrMany::Many(items)))
        }
    }

    /// Creates a collection from a hashbrown HashMap, failing if empty.
    ///
    /// This enables the JSON object syntax when the `hashbrown-json` feature is enabled.
    /// The HashMap's key-value pairs are collected into a vector of tuples.
    ///
    /// # Example
    /// ```rust
    /// # /// # {
    /// use cyrup_sugars::collections::OneOrMany;
    /// use hashbrown::HashMap;
    ///
    /// let map = hashbrown::hash_map! {
    ///     "key1" => "value1",
    ///     "key2" => "value2",
    /// };
    /// let collection: OneOrMany<(&str, &str)> = OneOrMany::from_hashmap(map)?;
    /// # }
    /// ```
    #[inline]
    pub fn from_hashmap<K, V>(
        map: ::hashbrown::HashMap<K, V>,
    ) -> Result<OneOrMany<(K, V)>, EmptyListError> {
        let items: Vec<(K, V)> = map.into_iter().collect();
        if items.is_empty() {
            Err(EmptyListError)
        } else {
            Ok(OneOrMany(ZeroOneOrMany::Many(items)))
        }
    }

    /// Creates a collection from a closure that returns a hashbrown HashMap, failing if empty.
    ///
    /// This enables the clean JSON object syntax when the `hashbrown-json` feature is enabled.
    /// This method is designed to work with the hashbrown macros.
    ///
    /// # Example
    /// ```rust
    /// # /// # {
    /// use cyrup_sugars::collections::OneOrMany;
    /// use cyrup_sugars::macros::hashbrown::hash_map_fn;
    ///
    /// let collection: OneOrMany<(&str, &str)> = OneOrMany::from_json(hash_map_fn! {
    ///     "beta" => "true",
    ///     "version" => "2.1.0",
    /// })?;
    /// # }
    /// ```
    #[inline]
    pub fn from_json<K, V, F>(f: F) -> Result<OneOrMany<(K, V)>, EmptyListError>
    where
        F: FnOnce() -> ::hashbrown::HashMap<K, V>,
    {
        Self::from_hashmap(f())
    }

    /// Merges multiple `OneOrMany`s into one, preserving order. Requires `T: Clone + 'static`.
    #[inline]
    pub fn merge<I>(items: I) -> Result<Self, EmptyListError>
    where
        I: IntoIterator<Item = OneOrMany<T>>,
        T: Clone + 'static,
    {
        let vec: Vec<T> = items
            .into_iter()
            .flat_map(|oom| oom.0.into_iter())
            .collect();
        Self::many(vec)
    }

    /// Merges references to multiple `OneOrMany`s into a new `OneOrMany<&T>`.
    #[inline]
    pub fn merge_refs<'a, I>(items: I) -> Result<OneOrMany<&'a T>, EmptyListError>
    where
        I: IntoIterator<Item = &'a OneOrMany<T>>,
    {
        let vec: Vec<&T> = items.into_iter().flat_map(|oom| oom.iter()).collect();
        OneOrMany::many(vec)
    }

    /// Returns the number of elements (always at least 1).
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the collection is empty (always false for OneOrMany).
    ///
    /// This method is provided for API consistency with other collection types,
    /// but OneOrMany always contains at least one element by construction.
    #[inline]
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Returns a reference to the first element.
    #[inline]
    pub fn first(&self) -> &T {
        // SAFETY: OneOrMany guarantees at least one element by construction
        match &self.0 {
            ZeroOneOrMany::None => unreachable!("OneOrMany cannot be None"),
            ZeroOneOrMany::One(item) => item,
            ZeroOneOrMany::Many(v) => &v[0],
        }
    }

    /// Returns a vector of references to all elements after the first.
    #[inline]
    pub fn rest(&self) -> Vec<&T> {
        self.0.rest()
    }

    /// Returns an iterator over references to all elements after the first.
    #[inline]
    pub fn rest_iter(&self) -> impl Iterator<Item = &T> {
        self.0.rest_iter()
    }

    /// Returns a new instance with an element added to the end.
    #[inline]
    pub fn with_pushed(self, item: T) -> Self {
        OneOrMany(self.0.with_pushed(item))
    }

    /// Returns a new instance with an element inserted at the specified index.
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub fn with_inserted(self, idx: usize, item: T) -> Self {
        OneOrMany(self.0.with_inserted(idx, item))
    }

    /// Maps each element to a new type, returning a new collection.
    #[inline]
    pub fn map<U, F: FnMut(T) -> U>(self, f: F) -> OneOrMany<U> {
        OneOrMany(self.0.map(f))
    }

    /// Maps each element to a new type, propagating errors.
    #[inline]
    pub fn try_map<U, E, F: FnMut(T) -> Result<U, E>>(self, f: F) -> Result<OneOrMany<U>, E> {
        self.0.try_map(f).map(OneOrMany)
    }

    /// Returns an iterator over references to the elements.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }
}

// Owned iterator requires T: Clone + 'static
impl<T: Clone + 'static> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = Box<dyn Iterator<Item = T>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.0.into_iter())
    }
}

// Borrowed iterator
impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = &'a T> + 'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

// Serde Support
impl<T: Serialize> Serialize for OneOrMany<T> {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            ZeroOneOrMany::None => unreachable!("OneOrMany cannot be None"),
            ZeroOneOrMany::One(item) => {
                let mut seq = ser.serialize_seq(Some(1))?;
                seq.serialize_element(item)?;
                seq.end()
            }
            ZeroOneOrMany::Many(v) => {
                let mut seq = ser.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
        }
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for OneOrMany<T> {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = OneOrMany<T>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a non-empty sequence or single value")
            }

            #[inline]
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(elem) = seq.next_element()? {
                    vec.push(elem);
                }
                if vec.is_empty() {
                    Err(de::Error::invalid_length(0, &"at least one element"))
                } else {
                    Ok(OneOrMany(ZeroOneOrMany::many(vec)))
                }
            }

            #[inline]
            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let v = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(OneOrMany(ZeroOneOrMany::One(v)))
            }
        }

        de.deserialize_any(V(PhantomData))
    }
}

// Conversion Traits
impl<T> From<T> for OneOrMany<T> {
    #[inline]
    fn from(value: T) -> Self {
        OneOrMany(ZeroOneOrMany::One(value))
    }
}

impl<T> TryFrom<Vec<T>> for OneOrMany<T> {
    type Error = EmptyListError;

    #[inline]
    fn try_from(vec: Vec<T>) -> Result<Self, Self::Error> {
        OneOrMany::many(vec)
    }
}

impl<T> From<OneOrMany<T>> for Vec<T> {
    #[inline]
    fn from(value: OneOrMany<T>) -> Self {
        value.0.into()
    }
}

impl<T> FromIterator<T> for OneOrMany<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        match iter.next() {
            None => panic!("OneOrMany requires at least one element"),
            Some(first) => {
                let rest: Vec<T> = iter.collect();
                if rest.is_empty() {
                    OneOrMany::one(first)
                } else {
                    let mut vec = Vec::with_capacity(1 + rest.len());
                    vec.push(first);
                    vec.extend(rest);
                    OneOrMany(ZeroOneOrMany::Many(vec))
                }
            }
        }
    }
}
