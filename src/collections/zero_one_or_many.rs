#[cfg(feature = "hashbrown-json")]
use serde::de::{self, Deserializer, MapAccess, SeqAccess, Visitor};
#[cfg(feature = "hashbrown-json")]
use serde::ser::{SerializeSeq, Serializer};
#[cfg(feature = "hashbrown-json")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "hashbrown-json")]
use std::fmt;
use std::iter::FromIterator;
#[cfg(feature = "hashbrown-json")]
use std::marker::PhantomData;

/// A collection that can hold zero, one, or many values of type `T`.
///
/// This enum provides an efficient way to represent collections that might be empty,
/// contain a single element, or contain multiple elements. It is designed to minimize
/// heap allocations and provide immutable operations that return new instances.
///
/// ### Variants
/// - `None`: Represents an empty collection with no elements. This variant uses zero
///   heap allocations.
/// - `One(T)`: Represents a collection with exactly one element. This variant also
///   uses zero heap allocations, as it directly holds the element.
/// - `Many(Vec<T>)`: Represents a collection with multiple elements. This variant
///   uses a `Vec<T>` to store the elements, with optimizations for pre-allocating
///   capacity when transitioning from `One` to `Many`.
///
/// ### Immutability
/// All operations on `ZeroOneOrMany` are immutable. Methods like `with_pushed` and
/// `with_inserted` consume the current instance and return a new instance with the
/// desired modifications. This ensures that the original collection remains unchanged,
/// promoting thread-safety and functional programming patterns.
///
/// ### Serialization and Deserialization
/// `ZeroOneOrMany<T>` implements `Serialize` and `Deserialize` from the Serde library.
/// - Serializes to a JSON array: `[]` for `None`, `[item]` for `One`, or multi-element
///   array for `Many`.
/// - Deserializes from `null`, a single value, or an array.
///
/// ### Performance
/// - **Zero Allocation**: `None` and `One` variants avoid heap allocations.
/// - **Pre-allocated Capacity**: Transitions from `One` to `Many` pre-allocate `Vec` capacity.
/// - **Inlined Methods**: Critical methods are marked `#[inline]` for performance.
/// - **Minimal Cloning**: Most operations do not require `T: Clone`.
///
/// ### Examples
/// ```rust
/// let empty = ZeroOneOrMany::none();
/// let single = ZeroOneOrMany::one(42);
/// let multiple = ZeroOneOrMany::many(vec![1, 2, 3]);
/// ```
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "hashbrown-json", derive(Serialize, Deserialize))]
pub enum ZeroOneOrMany<T> {
    /// Empty collection with zero elements.
    None,
    /// Collection containing exactly one element.
    One(T),
    /// Collection containing multiple elements stored in a Vec.
    Many(Vec<T>),
}

// Core API
impl<T> ZeroOneOrMany<T> {
    /// Creates an empty collection.
    #[inline]
    pub fn none() -> Self {
        ZeroOneOrMany::None
    }

    /// Creates a collection with a single element.
    #[inline]
    pub fn one(item: T) -> Self {
        ZeroOneOrMany::One(item)
    }

    /// Creates a collection from a `Vec<T>`, normalizing empty vectors to `None`.
    #[inline]
    pub fn many(items: Vec<T>) -> Self {
        if items.is_empty() {
            ZeroOneOrMany::None
        } else {
            ZeroOneOrMany::Many(items)
        }
    }

    /// Merges multiple `ZeroOneOrMany`s into one, preserving order. Requires `T: Clone + 'static`.
    #[inline]
    pub fn merge<I>(items: I) -> Self
    where
        I: IntoIterator<Item = ZeroOneOrMany<T>>,
        T: Clone + 'static,
    {
        let vec: Vec<T> = items
            .into_iter()
            .flat_map(|zoom| zoom.into_iter())
            .collect();
        Self::many(vec)
    }

    /// Returns the number of elements in the collection.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            ZeroOneOrMany::None => 0,
            ZeroOneOrMany::One(_) => 1,
            ZeroOneOrMany::Many(v) => v.len(),
        }
    }

    /// Checks if the collection is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, ZeroOneOrMany::None)
    }

    /// Returns a reference to the first element, if any.
    #[inline]
    pub fn first(&self) -> Option<&T> {
        match self {
            ZeroOneOrMany::None => None,
            ZeroOneOrMany::One(item) => Some(item),
            ZeroOneOrMany::Many(v) => v.first(),
        }
    }

    /// Returns a vector of references to all elements after the first.
    #[inline]
    pub fn rest(&self) -> Vec<&T> {
        match self {
            ZeroOneOrMany::None => vec![],
            ZeroOneOrMany::One(_) => vec![],
            ZeroOneOrMany::Many(v) if v.len() > 1 => v[1..].iter().collect(),
            ZeroOneOrMany::Many(_) => vec![],
        }
    }

    /// Returns an iterator over references to all elements after the first.
    #[inline]
    pub fn rest_iter(&self) -> impl Iterator<Item = &T> {
        match self {
            ZeroOneOrMany::None => [].iter(),
            ZeroOneOrMany::One(_) => [].iter(),
            ZeroOneOrMany::Many(v) if v.len() > 1 => v[1..].iter(),
            ZeroOneOrMany::Many(_) => [].iter(),
        }
    }

    /// Returns a new instance with an element added to the end.
    #[inline]
    pub fn with_pushed(self, item: T) -> Self {
        match self {
            ZeroOneOrMany::None => ZeroOneOrMany::One(item),
            ZeroOneOrMany::One(first) => {
                let mut vec = Vec::with_capacity(2);
                vec.push(first);
                vec.push(item);
                ZeroOneOrMany::Many(vec)
            }
            ZeroOneOrMany::Many(mut v) => {
                v.push(item);
                ZeroOneOrMany::Many(v)
            }
        }
    }

    /// Returns a new instance with an element inserted at the specified index.
    /// Panics if `idx` is out of bounds.
    #[inline]
    pub fn with_inserted(self, idx: usize, item: T) -> Self {
        match self {
            ZeroOneOrMany::None if idx == 0 => ZeroOneOrMany::One(item),
            ZeroOneOrMany::None => panic!("Index {} out of bounds for empty ZeroOneOrMany", idx),
            ZeroOneOrMany::One(first) if idx == 0 => {
                let mut vec = Vec::with_capacity(2);
                vec.push(item);
                vec.push(first);
                ZeroOneOrMany::Many(vec)
            }
            ZeroOneOrMany::One(first) if idx == 1 => {
                let mut vec = Vec::with_capacity(2);
                vec.push(first);
                vec.push(item);
                ZeroOneOrMany::Many(vec)
            }
            ZeroOneOrMany::One(_) => panic!(
                "Index {} out of bounds for single-element ZeroOneOrMany",
                idx
            ),
            ZeroOneOrMany::Many(mut v) => {
                v.insert(idx, item);
                ZeroOneOrMany::Many(v)
            }
        }
    }

    /// Maps each element to a new type, returning a new collection.
    #[inline]
    pub fn map<U, F: FnMut(T) -> U>(self, mut f: F) -> ZeroOneOrMany<U> {
        match self {
            ZeroOneOrMany::None => ZeroOneOrMany::None,
            ZeroOneOrMany::One(item) => ZeroOneOrMany::One(f(item)),
            ZeroOneOrMany::Many(v) => ZeroOneOrMany::Many(v.into_iter().map(f).collect()),
        }
    }

    /// Maps each element to a new type, propagating errors.
    #[inline]
    pub fn try_map<U, E, F: FnMut(T) -> Result<U, E>>(
        self,
        mut f: F,
    ) -> Result<ZeroOneOrMany<U>, E> {
        match self {
            ZeroOneOrMany::None => Ok(ZeroOneOrMany::None),
            ZeroOneOrMany::One(item) => Ok(ZeroOneOrMany::One(f(item)?)),
            ZeroOneOrMany::Many(v) => {
                let mut result = Vec::with_capacity(v.len());
                for item in v {
                    result.push(f(item)?);
                }
                Ok(ZeroOneOrMany::Many(result))
            }
        }
    }

    /// Returns an iterator over references to the elements.
    #[inline]
    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        match self {
            ZeroOneOrMany::None => Box::new([].iter()),
            ZeroOneOrMany::One(item) => Box::new(std::iter::once(item)),
            ZeroOneOrMany::Many(v) => Box::new(v.iter()),
        }
    }
}

// Owned iterator requires T: Clone
impl<T: Clone + 'static> IntoIterator for ZeroOneOrMany<T> {
    type Item = T;
    type IntoIter = Box<dyn Iterator<Item = T>>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            ZeroOneOrMany::None => Box::new(std::iter::empty()),
            ZeroOneOrMany::One(item) => Box::new(std::iter::once(item)),
            ZeroOneOrMany::Many(v) => Box::new(v.into_iter()),
        }
    }
}

// Borrowed iterator
impl<'a, T> IntoIterator for &'a ZeroOneOrMany<T> {
    type Item = &'a T;
    type IntoIter = Box<dyn Iterator<Item = &'a T> + 'a>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Box::new(self.iter())
    }
}

// Serde Support - already handled by derive macro
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        match self {
            ZeroOneOrMany::None => {
                let seq = ser.serialize_seq(Some(0))?;
                seq.end()
            }
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

// Deserialize already handled by derive macro
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        struct V<T>(PhantomData<T>);
        impl<'de, T: Deserialize<'de>> Visitor<'de> for V<T> {
            type Value = ZeroOneOrMany<T>;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("null, a sequence, or single value")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ZeroOneOrMany::None)
            }

            #[inline]
            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ZeroOneOrMany::None)
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
                Ok(ZeroOneOrMany::many(vec))
            }

            #[inline]
            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let v = Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(ZeroOneOrMany::One(v))
            }
        }

        de.deserialize_any(V(PhantomData))
    }
}

// Conversion Traits
impl<T> From<T> for ZeroOneOrMany<T> {
    #[inline]
    fn from(value: T) -> Self {
        ZeroOneOrMany::One(value)
    }
}

impl<T> From<Vec<T>> for ZeroOneOrMany<T> {
    #[inline]
    fn from(vec: Vec<T>) -> Self {
        ZeroOneOrMany::many(vec)
    }
}

impl<T> FromIterator<T> for ZeroOneOrMany<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ZeroOneOrMany::many(iter.into_iter().collect())
    }
}

impl<T> From<ZeroOneOrMany<T>> for Vec<T> {
    #[inline]
    fn from(value: ZeroOneOrMany<T>) -> Self {
        match value {
            ZeroOneOrMany::None => vec![],
            ZeroOneOrMany::One(item) => vec![item],
            ZeroOneOrMany::Many(v) => v,
        }
    }
}
