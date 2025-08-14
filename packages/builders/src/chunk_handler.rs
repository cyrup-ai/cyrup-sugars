//! Generic chunk and error handling trait for builders
//!
//! This module provides the `ChunkHandler` trait that any builder can implement
//! to get `.on_chunk()` and `.on_error()` methods with their own return types.

/// Trait for builders that can handle chunks and errors with custom handlers
pub trait ChunkHandler<T>: Sized {
    /// Set a handler for successful chunks
    /// 
    /// # Example
    /// ```rust
    /// builder.on_chunk(|chunk| {
    ///     println!("Processing: {}", chunk);
    ///     chunk.into()
    /// })
    /// ```
    fn on_chunk<F>(self, handler: F) -> Self
    where
        F: Fn(T) -> T + Send + Sync + 'static;

    /// Set a handler for error chunks
    /// 
    /// # Example
    /// ```rust
    /// builder.on_error(|error| {
    ///     eprintln!("Error: {}", error);
    ///     T::default() // or convert error to T
    /// })
    /// ```
    fn on_error<F>(self, handler: F) -> Self
    where
        F: Fn(String) -> T + Send + Sync + 'static;
}