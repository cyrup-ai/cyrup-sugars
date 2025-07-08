use super::zero_one_or_many::ZeroOneOrMany;
#[cfg(feature = "hashbrown-json")]
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "hashbrown-json")]
use serde::ser::{SerializeSeq, Serializer};
#[cfg(feature = "hashbrown-json")]
use serde::{Deserialize, Serialize};
use std::fmt;
use std::iter::FromIterator;
#[cfg(feature = "hashbrown-json")]
use std::marker::PhantomData;

/// A non-empty collection that holds one or many values of type `T`.
///
/// This struct wraps `ZeroOneOrMany<T>` to ensure it always contains at least one element.
/// Attempts to create an empty `OneOrMany` result in an `EmptyListError`.
///
/// ### Immutability
/// All operations are immutable, returning new instances while preserving the non-empty invariant.
///
/// ### Serialization and Deserialization
/// Serializes to a JSON array with at least one element. Deserialization fails on `null` or empty arrays.
///
/// ### Performance
/// - **Zero Allocation**: Reuses allocations from `ZeroOneOrMany<T>` where possible.
/// - **Inlined Methods**: Critical methods are optimized with `#[inline]`.
/// - **Minimal Cloning**: Most operations avoid requiring `T: Clone`.
///
/// ### Examples
/// ```rust
/// let single = OneOrMany::one(42);
/// let multiple = OneOrMany::many(vec![1, 2, 3]).unwrap();
/// ```
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "hashbrown-json", derive(Serialize, Deserialize))]
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

    /// Merges multiple `OneOrMany`s into one. Requires `T: Clone + 'static`.
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

    /// Returns the number of elements (always at least 1).
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a reference to the first element.
    #[inline]
    pub fn first(&self) -> &T {
        self.0
            .first()
            .expect("OneOrMany always has at least one element")
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

    /// Maps each element to a new type using a closure.
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

// Owned iterator requires T: Clone
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

// Serde Support is handled by derive macro on the struct

#[cfg(feature = "hashbrown-json")]
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
                    Ok(OneOrMany(ZeroOneOrMany::Many(vec)))
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
        let vec: Vec<T> = iter.into_iter().collect();
        OneOrMany::many(vec).expect("OneOrMany cannot be constructed from empty iterator")
    }
}
