pub mod agent_builder;
pub mod macros;
pub mod models;

// Re-export the hash_map_fn! macro so [("key", "value")] syntax works
pub use sugars_macros::hash_map_fn;

// Re-export the hash_map macro for array tuple syntax
pub use sugars_collections::hash_map;

/// Macro that automatically handles array tuple syntax in builder patterns
/// This is pushed down into the builder implementation, not visible to users
#[macro_export]
macro_rules! array_tuple_closure_llm {
    // Transform the entire builder chain using array_tuple_closure macro
    ($($tokens:tt)*) => {
        sugars_collections::array_tuple_closure! {
            $($tokens)*
        }
    };
}

// Re-export the FluentAi builder and all required types
pub use agent_builder::{
    Agent, AgentRoleBuilder, Context, ConversationChunk, Directory, File, Files, FluentAi, Github,
    Library, MessageRole, NamedTool, Perplexity, Stdio, Tool, exec_to_text,
};

// Re-export the array_tuple_closure macro for transparent array tuple syntax
pub use sugars_collections::array_tuple_closure;

// Re-export models for convenient access
pub use models::*;

// The hash_map_fn! macro is pushed down INTO the builder
// Users don't see it - they just use [("key", "value")] syntax
