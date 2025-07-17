//! FluentAI main builder interface
//!
//! This module provides the main FluentAI entry point that delegates to domain builders.

use crate::domain::agent_role::AgentRoleBuilder;

/// Master builder for Fluent AI - semantic entry point for all builders
pub struct FluentAi;

impl FluentAi {
    /// Create an agent role with persistent context and tools
    /// EXACT syntax: FluentAi::agent_role("rusty-squire")
    pub fn agent_role(name: impl Into<String>) -> AgentRoleBuilder {
        AgentRoleBuilder::new(name)
    }
}