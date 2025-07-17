//! Async stream with built-in error handling and collection support
//!
//! This module provides feature-gated implementations for different async runtimes:
//! - tokio-async: Uses tokio::sync::mpsc for Tokio ecosystem
//! - std-async: Uses async-channel (runtime-agnostic)  
//! - crossbeam-async: Uses async-channel + crossbeam for compute-heavy workloads

#[cfg(feature = "tokio-backend")]
pub mod tokio;
#[cfg(feature = "tokio-backend")]
pub use tokio::AsyncStream;

#[cfg(all(feature = "std-backend", not(feature = "tokio-backend")))]
pub mod std;
#[cfg(all(feature = "std-backend", not(feature = "tokio-backend")))]
pub use std::AsyncStream;

#[cfg(all(
    feature = "crossbeam-backend",
    not(feature = "tokio-backend"),
    not(feature = "std-backend")
))]
pub mod crossbeam;
#[cfg(all(
    feature = "crossbeam-backend",
    not(feature = "tokio-backend"),
    not(feature = "std-backend")
))]
pub use crossbeam::AsyncStream;
