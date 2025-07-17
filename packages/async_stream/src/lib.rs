//! Asynchronous programming utilities
//!
//! Choose your runtime and get both AsyncTask and AsyncStream plus convenient macros:
//! - `tokio-async`: Full Tokio ecosystem with mpsc channels
//! - `std-async`: Runtime-agnostic using async-channel  
//! - `crossbeam-async`: Compute-heavy workloads with crossbeam + async-channel

pub mod emitter_builder;
pub mod result_types;
pub mod stream;
pub mod stream_ext;

// Runtime-specific unified exports
#[cfg(all(
    feature = "crossbeam-backend",
    not(feature = "tokio-backend"),
    not(feature = "std-backend")
))]
pub use stream::crossbeam::AsyncStream;
#[cfg(all(feature = "std-backend", not(feature = "tokio-backend")))]
pub use stream::std::AsyncStream;
#[cfg(feature = "tokio-backend")]
pub use stream::tokio::AsyncStream;

// Core types available in all configurations
pub use emitter_builder::{EmitterBuilder, EmitterImpl};
pub use result_types::{AsyncResult, AsyncResultChunk};
pub use stream_ext::StreamExt;

// Re-export from async_task
pub use sugars_async_task::{AsyncTask, NotResult};

/// Pipe operator for fluent chaining of operations
#[macro_export]
macro_rules! pipe {
    ($value:expr => $func:expr) => {
        $func($value)
    };
    ($value:expr => $func:expr => $($rest:tt)*) => {
        pipe!($func($value) => $($rest)*)
    };
}
