pub mod agent_builder;
pub mod macros;
pub mod models;

// Re-export the hash_map macro for JSON syntax
pub use sugars_collections::hash_map;

// Removed broken macros - using existing json_closure! infrastructure instead

// Re-export the FluentAi builder and all required types
pub use agent_builder::{
    Agent, AgentRoleBuilder, BadChunk, ChatInput, ChatLoop, Context, Directory, File, Files,
    FluentAi, Github, Library, MessageChunk, MessageRole, Mistral, Models, NamedTool, Perplexity,
    Providers, Stdio, Tool, exec_to_text, log,
};

// Re-export the JSON closure macro for transparent JSON syntax
pub use sugars_collections::json_closure;

// Re-export models for convenient access
pub use models::*;

// The hash_map_fn! macro is pushed down INTO the builder
// Users don't see it - they just use {"key" => "value"} syntax
