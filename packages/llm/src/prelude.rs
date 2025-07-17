//! Prelude module that brings JSON syntax into scope

// This brings the hash_map_fn! macro into scope so {"key" => "value"} syntax works
pub use sugars_macros::hash_map_fn;

// Re-export all builder types
pub use crate::agent_builder::*;
pub use crate::models::*;