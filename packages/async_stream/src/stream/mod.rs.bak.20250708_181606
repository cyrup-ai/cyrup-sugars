//! Async stream with built-in error handling and collection support
//!
//! This module provides feature-gated implementations for different async runtimes:
//! - tokio-async: Uses tokio::sync::mpsc for Tokio ecosystem
//! - std-async: Uses async-channel (runtime-agnostic)  
//! - crossbeam-async: Uses async-channel + crossbeam for compute-heavy workloads

#[cfg(feature = "tokio-async")]
pub mod tokio;
#[cfg(feature = "tokio-async")]
pub use tokio::AsyncStream;

#[cfg(all(feature = "std-async", not(feature = "tokio-async")))]
pub mod std;
#[cfg(all(feature = "std-async", not(feature = "tokio-async")))]
pub use std::AsyncStream;

#[cfg(all(
    feature = "crossbeam-async",
    not(feature = "tokio-async"),
    not(feature = "std-async")
))]
pub mod crossbeam;
#[cfg(all(
    feature = "crossbeam-async",
    not(feature = "tokio-async"),
    not(feature = "std-async")
))]
pub use crossbeam::AsyncStream;
