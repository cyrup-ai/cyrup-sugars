//! Model provider implementations
//!
//! This module defines the completion model providers like Mistral, Anthropic, etc.

/// Mistral model provider
pub struct Mistral;

impl Mistral {
    pub const MAGISTRAL_SMALL: Self = Self;
}

/// Anthropic model provider  
pub struct Anthropic;

impl Anthropic {
    pub const CLAUDE4_SONNET: Self = Self;
}

/// Standard I/O marker type for MCP servers
pub struct Stdio;
