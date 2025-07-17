//! FluentAI Builder Implementation
//! 
//! This module provides the FluentAI builder interface that delegates to the existing
//! AgentRoleBuilder which already has all methods implemented.

use crate::domain::agent_role::AgentRoleBuilder;

/// Master builder for Fluent AI - delegates to AgentRoleBuilder
pub struct FluentAi;

impl FluentAi {
    /// Create an agent role with persistent context and tools
    /// EXACT syntax: FluentAi::agent_role("rusty-squire")
    /// 
    /// This delegates to AgentRoleBuilder which already has all methods:
    /// - completion_provider, temperature, max_tokens, system_prompt
    /// - context, mcp_server, tools, additional_params, metadata  
    /// - on_tool_result, on_conversation_turn, on_chunk
    /// - into_agent, conversation_history, chat
    pub fn agent_role(name: impl Into<String>) -> AgentRoleBuilder {
        AgentRoleBuilder::new(name)
    }
}