//! Result types that are allowed in AsyncTask and AsyncStream
//!
//! These special Result types implement NotResult to bypass the negative impl restriction,
//! allowing error handling within async operations while maintaining the "always unwrapped" pattern.

use super::task::NotResult;

/// A Result type that can be used with AsyncTask
///
/// This type bypasses the negative impl restriction on Result types,
/// allowing error handling within AsyncTask operations.
#[derive(Debug)]
pub struct AsyncResult<T, E> {
    inner: Result<T, E>,
}

impl<T, E> AsyncResult<T, E> {
    /// Creates a new AsyncResult with a success value.
    pub fn ok(value: T) -> Self {
        Self { inner: Ok(value) }
    }

    /// Creates a new async result with an error value.
    pub fn err(error: E) -> Self {
        Self { inner: Err(error) }
    }

    /// Consumes the async result and returns the inner Result.
    pub fn into_inner(self) -> Result<T, E> {
        self.inner
    }

    /// Returns a reference to the inner Result's value or error.
    pub fn as_ref(&self) -> Result<&T, &E> {
        self.inner.as_ref()
    }

    /// Returns true if the async result contains a success value.
    pub fn is_ok(&self) -> bool {
        self.inner.is_ok()
    }

    /// Returns true if the async result contains an error value.
    pub fn is_err(&self) -> bool {
        self.inner.is_err()
    }
}

// Explicitly implement NotResult for AsyncResult
impl<T, E> NotResult for AsyncResult<T, E> {}

/// A Result type for streaming chunks that can be used with AsyncStream
///
/// This type is designed for streaming operations where each chunk
/// might succeed or fail independently.
#[derive(Debug)]
pub struct AsyncResultChunk<T, E> {
    inner: Result<T, E>,
}

impl<T, E> AsyncResultChunk<T, E> {
    /// Creates a new AsyncResultChunk with a success value.
    pub fn ok(value: T) -> Self {
        Self { inner: Ok(value) }
    }

    /// Creates a new async result with an error value.
    pub fn err(error: E) -> Self {
        Self { inner: Err(error) }
    }

    /// Consumes the async result and returns the inner Result.
    pub fn into_inner(self) -> Result<T, E> {
        self.inner
    }

    /// Returns a reference to the inner Result's value or error.
    pub fn as_ref(&self) -> Result<&T, &E> {
        self.inner.as_ref()
    }

    /// Returns true if the async result contains a success value.
    pub fn is_ok(&self) -> bool {
        self.inner.is_ok()
    }

    /// Returns true if the async result contains an error value.
    pub fn is_err(&self) -> bool {
        self.inner.is_err()
    }
}

// Explicitly implement NotResult for AsyncResultChunk
impl<T, E> NotResult for AsyncResultChunk<T, E> {}

// Conversion traits
impl<T, E> From<Result<T, E>> for AsyncResult<T, E> {
    fn from(result: Result<T, E>) -> Self {
        Self { inner: result }
    }
}

impl<T, E> From<Result<T, E>> for AsyncResultChunk<T, E> {
    fn from(result: Result<T, E>) -> Self {
        Self { inner: result }
    }
}
