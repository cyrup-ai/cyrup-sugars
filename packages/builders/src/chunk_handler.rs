//! Generic chunk handling traits for builders
//!
//! This module provides traits for handling message chunks in builders:
//! - `MessageChunk` - trait for types that can represent both success and error states
//! - `ChunkHandler` - trait for handling streaming Results

/// Trait for message chunks that can represent both success and error states
pub trait MessageChunk: Sized {
    /// Create a bad chunk from an error
    fn bad_chunk(error: String) -> Self;

    /// Get the error if this is a bad chunk
    fn error(&self) -> Option<&str>;

    /// Check if this chunk represents an error
    fn is_error(&self) -> bool {
        self.error().is_some()
    }
}

/// Trait for builders that can handle streaming Results by unwrapping them
///
/// The `on_chunk` method takes a `Result<T, E>` and returns `T`, unwrapping the Result.
pub trait ChunkHandler<T, E = String>: Sized
where
    T: MessageChunk,
{
    /// Set a handler that unwraps Result<T, E> to T
    ///
    /// # Example
    /// ```ignore
    /// builder.on_chunk(|result| match result {
    ///     Ok(chunk) => {
    ///         println!("Processing: {}", chunk);
    ///         chunk
    ///     },
    ///     Err(e) => {
    ///         eprintln!("Error: {}", e);
    ///         T::bad_chunk(e)
    ///     }
    /// })
    /// ```
    fn on_chunk<F>(self, handler: F) -> Self
    where
        F: Fn(Result<T, E>) -> T + Send + Sync + 'static;
}
