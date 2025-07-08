//! Asynchronous programming utilities
//!
//! Choose your runtime and get both AsyncTask and AsyncStream plus convenient macros:
//! - `tokio-async`: Full Tokio ecosystem with mpsc channels
//! - `std-async`: Runtime-agnostic using async-channel  
//! - `crossbeam-async`: Compute-heavy workloads with crossbeam + async-channel

pub mod emitter_builder;
pub mod future_ext;
pub mod result_types;
pub mod stream;
pub mod stream_ext;
pub mod task;

// Runtime-specific unified exports
#[cfg(all(
    feature = "crossbeam-async",
    not(feature = "tokio-async"),
    not(feature = "std-async")
))]
pub use stream::crossbeam::AsyncStream;
#[cfg(all(feature = "std-async", not(feature = "tokio-async")))]
pub use stream::std::AsyncStream;
#[cfg(feature = "tokio-async")]
pub use stream::tokio::AsyncStream;

// Core types available in all configurations
pub use emitter_builder::{EmitterBuilder, EmitterImpl};
pub use result_types::{AsyncResult, AsyncResultChunk};
pub use task::{AsyncTask, NotResult};

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
