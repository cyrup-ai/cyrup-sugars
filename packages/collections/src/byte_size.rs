//! Byte size utilities for semantic sizing

#[cfg(feature = "array-tuples")]
use serde::{Deserialize, Serialize};

/// Represents a size in bytes with semantic constructors
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "array-tuples", derive(Serialize, Deserialize))]
pub struct ByteSize(pub usize);

impl ByteSize {
    /// Create a new ByteSize from bytes
    pub fn bytes(size: usize) -> Self {
        Self(size)
    }

    /// Create a new ByteSize from kilobytes
    pub fn kilobytes(size: usize) -> Self {
        Self(size * 1024)
    }

    /// Create a new ByteSize from megabytes
    pub fn megabytes(size: usize) -> Self {
        Self(size * 1024 * 1024)
    }

    /// Get the size in bytes
    pub fn as_bytes(&self) -> usize {
        self.0
    }
}

impl From<usize> for ByteSize {
    fn from(bytes: usize) -> Self {
        Self(bytes)
    }
}

impl From<ByteSize> for usize {
    fn from(size: ByteSize) -> Self {
        size.0
    }
}

impl std::ops::Add for ByteSize {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        ByteSize(self.0 + other.0)
    }
}

/// Extension trait to add .bytes() method to integers
pub trait ByteSizeExt {
    /// Converts this value to a ByteSize representing the given number of bytes.
    fn bytes(self) -> ByteSize;
    /// Converts this value to a ByteSize representing the given number of kilobytes.
    fn kb(self) -> ByteSize;
    /// Converts this value to a ByteSize representing the given number of megabytes.
    fn mb(self) -> ByteSize;
}

impl ByteSizeExt for usize {
    fn bytes(self) -> ByteSize {
        ByteSize::bytes(self)
    }

    fn kb(self) -> ByteSize {
        ByteSize::kilobytes(self)
    }

    fn mb(self) -> ByteSize {
        ByteSize::megabytes(self)
    }
}

impl ByteSizeExt for u32 {
    fn bytes(self) -> ByteSize {
        ByteSize::bytes(self as usize)
    }

    fn kb(self) -> ByteSize {
        ByteSize::kilobytes(self as usize)
    }

    fn mb(self) -> ByteSize {
        ByteSize::megabytes(self as usize)
    }
}

impl ByteSizeExt for u64 {
    fn bytes(self) -> ByteSize {
        ByteSize::bytes(self as usize)
    }

    fn kb(self) -> ByteSize {
        ByteSize::kilobytes(self as usize)
    }

    fn mb(self) -> ByteSize {
        ByteSize::megabytes(self as usize)
    }
}

impl ByteSizeExt for i32 {
    fn bytes(self) -> ByteSize {
        ByteSize::bytes(self as usize)
    }

    fn kb(self) -> ByteSize {
        ByteSize::kilobytes(self as usize)
    }

    fn mb(self) -> ByteSize {
        ByteSize::megabytes(self as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_byte_size_constructors() {
        assert_eq!(ByteSize::bytes(1024), ByteSize(1024));
        assert_eq!(ByteSize::kilobytes(1), ByteSize(1024));
        assert_eq!(ByteSize::megabytes(1), ByteSize(1024 * 1024));
    }

    #[test]
    fn test_byte_size_zero() {
        assert_eq!(ByteSize::bytes(0), ByteSize(0));
        assert_eq!(ByteSize::kilobytes(0), ByteSize(0));
        assert_eq!(ByteSize::megabytes(0), ByteSize(0));
    }

    #[test]
    fn test_byte_size_large_values() {
        assert_eq!(ByteSize::bytes(usize::MAX), ByteSize(usize::MAX));
        // Test large kilobytes without overflow
        let large_kb = usize::MAX / 1024;
        assert_eq!(ByteSize::kilobytes(large_kb), ByteSize(large_kb * 1024));
        // Test large megabytes without overflow
        let large_mb = usize::MAX / (1024 * 1024);
        assert_eq!(
            ByteSize::megabytes(large_mb),
            ByteSize(large_mb * 1024 * 1024)
        );
    }

    #[test]
    fn test_byte_size_ext_usize() {
        assert_eq!(512.bytes(), ByteSize(512));
        assert_eq!(1.kb(), ByteSize(1024));
        assert_eq!(1.mb(), ByteSize(1024 * 1024));
        assert_eq!(0.bytes(), ByteSize(0));
        assert_eq!(usize::MAX.bytes(), ByteSize(usize::MAX));
    }

    #[test]
    fn test_byte_size_ext_u32() {
        assert_eq!(512u32.bytes(), ByteSize(512));
        assert_eq!(1u32.kb(), ByteSize(1024));
        assert_eq!(1u32.mb(), ByteSize(1024 * 1024));
        assert_eq!(0u32.bytes(), ByteSize(0));
        assert_eq!(u32::MAX.bytes(), ByteSize(u32::MAX as usize));
    }

    #[test]
    fn test_byte_size_ext_u64() {
        assert_eq!(512u64.bytes(), ByteSize(512));
        assert_eq!(1u64.kb(), ByteSize(1024));
        assert_eq!(1u64.mb(), ByteSize(1024 * 1024));
        assert_eq!(0u64.bytes(), ByteSize(0));
        // Test with large u64 values that fit in usize
        let large_val = usize::MAX as u64;
        assert_eq!(large_val.bytes(), ByteSize(large_val as usize));
    }

    #[test]
    fn test_byte_size_ext_i32() {
        assert_eq!(512i32.bytes(), ByteSize(512));
        assert_eq!(1i32.kb(), ByteSize(1024));
        assert_eq!(1i32.mb(), ByteSize(1024 * 1024));
        assert_eq!(0i32.bytes(), ByteSize(0));
        assert_eq!(i32::MAX.bytes(), ByteSize(i32::MAX as usize));
    }

    #[test]
    fn test_from_usize() {
        let size = ByteSize::from(2048);
        assert_eq!(size, ByteSize(2048));
        assert_eq!(size.as_bytes(), 2048);
    }

    #[test]
    fn test_into_usize() {
        let size = ByteSize::bytes(2048);
        let bytes: usize = size.into();
        assert_eq!(bytes, 2048);
    }

    #[test]
    fn test_as_bytes() {
        assert_eq!(ByteSize::bytes(0).as_bytes(), 0);
        assert_eq!(ByteSize::bytes(1024).as_bytes(), 1024);
        assert_eq!(ByteSize::kilobytes(2).as_bytes(), 2048);
        assert_eq!(ByteSize::megabytes(1).as_bytes(), 1024 * 1024);
    }

    #[test]
    fn test_ordering() {
        let small = ByteSize::bytes(100);
        let medium = ByteSize::kilobytes(1);
        let large = ByteSize::megabytes(1);

        assert!(small < medium);
        assert!(medium < large);
        assert!(small < large);

        assert_eq!(small, ByteSize::bytes(100));
        assert_ne!(small, medium);
    }

    #[test]
    fn test_clone_copy() {
        let original = ByteSize::kilobytes(5);
        #[allow(clippy::clone_on_copy)]
        let cloned = original.clone();
        let copied = original;

        assert_eq!(original, cloned);
        assert_eq!(original, copied);
        assert_eq!(cloned, copied);
    }

    #[test]
    fn test_debug_format() {
        let size = ByteSize::bytes(1024);
        let debug_str = format!("{:?}", size);
        assert_eq!(debug_str, "ByteSize(1024)");
    }

    #[cfg(feature = "array-tuples")]
    #[test]
    fn test_serde_serialization() {
        use serde_json;
        let size = ByteSize::megabytes(5);
        let serialized = serde_json::to_string(&size).expect("test serialization");
        let deserialized: ByteSize =
            serde_json::from_str(&serialized).expect("test deserialization");
        assert_eq!(size, deserialized);
    }

    #[cfg(feature = "array-tuples")]
    #[test]
    fn test_serde_zero() {
        use serde_json;
        let size = ByteSize::bytes(0);
        let serialized = serde_json::to_string(&size).expect("test serialization");
        let deserialized: ByteSize =
            serde_json::from_str(&serialized).expect("test deserialization");
        assert_eq!(size, deserialized);
    }

    #[cfg(feature = "array-tuples")]
    #[test]
    fn test_serde_large() {
        use serde_json;
        let size = ByteSize::bytes(usize::MAX);
        let serialized = serde_json::to_string(&size).expect("test serialization");
        let deserialized: ByteSize =
            serde_json::from_str(&serialized).expect("test deserialization");
        assert_eq!(size, deserialized);
    }

    #[test]
    fn test_units_conversion() {
        // Test exact conversions
        assert_eq!(1.kb().as_bytes(), 1024);
        assert_eq!(2.kb().as_bytes(), 2048);
        assert_eq!(1.mb().as_bytes(), 1024 * 1024);
        assert_eq!(2.mb().as_bytes(), 2 * 1024 * 1024);

        // Test mixed operations
        let total = 1.mb() + 512.kb() + 256.bytes();
        assert_eq!(total.as_bytes(), 1024 * 1024 + 512 * 1024 + 256);
    }

    #[test]
    fn test_realistic_file_sizes() {
        // Test realistic file sizes
        let small_file = 4.kb(); // Small text file
        let image = 2.mb(); // Medium image
        let video = 100.mb(); // Small video

        assert_eq!(small_file.as_bytes(), 4 * 1024);
        assert_eq!(image.as_bytes(), 2 * 1024 * 1024);
        assert_eq!(video.as_bytes(), 100 * 1024 * 1024);

        assert!(small_file < image);
        assert!(image < video);
    }
}
