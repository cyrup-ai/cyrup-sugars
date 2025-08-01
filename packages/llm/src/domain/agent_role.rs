//! Agent role builder implementation following ARCHITECTURE.md exactly

use serde_json::Value;
use cyrup_sugars::{AsyncStream, ZeroOneOrMany};
use crate::domain::{MessageRole};
use crate::domain::chunk::ChatMessageChunk;
use std::collections::HashMap;
use std::marker::PhantomData;

/// Trait for converting various types to HashMaps for JSON-like syntax support
pub trait IntoHashMap {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str>;
}

/// Implement IntoHashMap for closures (existing functionality)
impl<F> IntoHashMap for F 
where
    F: FnOnce() -> hashbrown::HashMap<&'static str, &'static str>
{
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self()
    }
}

/// Implement IntoHashMap for direct HashMap (zero-copy for pre-built maps)
impl IntoHashMap for hashbrown::HashMap<&'static str, &'static str> {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self
    }
}

/// Implement IntoHashMap for array of tuples (compile-time JSON-like syntax)
impl<const N: usize> IntoHashMap for [(&'static str, &'static str); N] {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self.into_iter().collect()
    }
}

/// Implement IntoHashMap for Vec of tuples (runtime JSON-like syntax)
impl IntoHashMap for Vec<(&'static str, &'static str)> {
    fn into_hashmap(self) -> hashbrown::HashMap<&'static str, &'static str> {
        self.into_iter().collect()
    }
}


/// Builder for creating agent roles - EXACT API from ARCHITECTURE.md
pub struct AgentRoleBuilder {
    name: String,
    completion_provider: Option<Box<dyn std::any::Any + Send + Sync>>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    system_prompt: Option<String>,
    contexts: Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>,
    tools: Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>,
    mcp_servers: Option<ZeroOneOrMany<McpServerConfig>>,
    additional_params: Option<HashMap<String, Value>>,
    memory: Option<Box<dyn std::any::Any + Send + Sync>>,
    metadata: Option<HashMap<String, Value>>,
    on_tool_result_handler: Option<Box<dyn Fn(ZeroOneOrMany<Value>) + Send + Sync>>,
    on_conversation_turn_handler: Option<Box<dyn Fn(&AgentConversation, &AgentRoleAgent) + Send + Sync>>,
}

/// MCP Server configuration
struct McpServerConfig {
    server_type: String,
    bin_path: Option<String>,
    init_command: Option<String>,
}

/// MCP Server builder
pub struct McpServerBuilder<T> {
    parent: AgentRoleBuilder,
    server_type: PhantomData<T>,
    bin_path: Option<String>,
}

/// Placeholder for Stdio type
pub struct Stdio;

/// Agent type placeholder for agent role
pub struct AgentRoleAgent;

/// Agent conversation type
pub struct AgentConversation {
    messages: Option<ZeroOneOrMany<(MessageRole, String)>>,
}

impl AgentConversation {
    pub fn last(&self) -> AgentConversationMessage {
        AgentConversationMessage {
            content: self.messages.as_ref()
                .and_then(|msgs| {
                    // Get the last element from ZeroOneOrMany
                    let all: Vec<_> = msgs.clone().into_iter().collect();
                    all.last().map(|(_, m)| m.clone())
                })
                .unwrap_or_default(),
        }
    }
}

pub struct AgentConversationMessage {
    content: String,
}

impl AgentConversationMessage {
    pub fn message(&self) -> &str {
        &self.content
    }
}


impl AgentRoleBuilder {
    /// Create a new agent role builder
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            completion_provider: None,
            temperature: None,
            max_tokens: None,
            system_prompt: None,
            contexts: None,
            tools: None,
            mcp_servers: None,
            additional_params: None,
            memory: None,
            metadata: None,
            on_tool_result_handler: None,
            on_conversation_turn_handler: None,
        }
    }
    
    /// Set the completion provider - EXACT syntax: .completion_provider(Mistral::MagistralSmall)
    pub fn completion_provider(mut self, provider: impl std::any::Any + Send + Sync + 'static) -> Self {
        self.completion_provider = Some(Box::new(provider));
        self
    }
    
    /// Set temperature - EXACT syntax: .temperature(1.0)
    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }
    
    /// Set max tokens - EXACT syntax: .max_tokens(8000)
    pub fn max_tokens(mut self, max: u64) -> Self {
        self.max_tokens = Some(max);
        self
    }
    
    /// Set system prompt - EXACT syntax: .system_prompt("...")
    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
    
    /// Add context - EXACT syntax: .context(Context<File>::of(...), Context<Files>::glob(...), ...)
    pub fn context(mut self, contexts: impl ContextArgs) -> Self {
        contexts.add_to(&mut self.contexts);
        self
    }
    
    /// Add MCP server - EXACT syntax: .mcp_server<Stdio>::bin(...).init(...)
    pub fn mcp_server<T>(self) -> McpServerBuilder<T> {
        McpServerBuilder {
            parent: self,
            server_type: PhantomData,
            bin_path: None,
        }
    }
    
    /// Add tools - EXACT syntax: .tools(Tool<Perplexity>::new(...), Tool::named(...).bin(...).description(...))
    pub fn tools(mut self, tools: impl ToolArgs) -> Self {
        tools.add_to(&mut self.tools);
        self
    }
    
    /// Set additional parameters - EXACT syntax: .additional_params({"beta" => "true"})
    pub fn additional_params<T>(mut self, params: T) -> Self
    where
        T: IntoHashMap
    {
        let config_map = params.into_hashmap();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        self.additional_params = Some(map);
        self
    }
    
    /// Set memory - EXACT syntax: .memory(Library::named("obsidian_vault"))
    pub fn memory(mut self, memory: impl std::any::Any + Send + Sync + 'static) -> Self {
        self.memory = Some(Box::new(memory));
        self
    }
    
    /// Set metadata - EXACT syntax: .metadata({"key" => "val", "foo" => "bar"})
    pub fn metadata<T>(mut self, meta: T) -> Self
    where
        T: IntoHashMap
    {
        let config_map = meta.into_hashmap();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        self.metadata = Some(map);
        self
    }
    
    
    /// Set on_tool_result handler - EXACT syntax: .on_tool_result(|results| { ... })
    pub fn on_tool_result<F>(mut self, handler: F) -> Self
    where
        F: Fn(ZeroOneOrMany<Value>) + Send + Sync + 'static,
    {
        self.on_tool_result_handler = Some(Box::new(handler));
        self
    }
    
    /// Set on_conversation_turn handler - EXACT syntax: .on_conversation_turn(|conversation, agent| { ... })
    pub fn on_conversation_turn<F>(mut self, handler: F) -> Self
    where
        F: Fn(&AgentConversation, &AgentRoleAgent) + Send + Sync + 'static,
    {
        self.on_conversation_turn_handler = Some(Box::new(handler));
        self
    }
    
    /// Set chunk handler - EXACT syntax: .on_chunk(|chunk| { ... })
    /// MUST precede .chat()
    pub fn on_chunk<F>(self, handler: F) -> AgentRoleBuilderWithChunkHandler
    where
        F: Fn(Result<ChatMessageChunk, String>) -> Result<ChatMessageChunk, String> + Send + Sync + 'static,
    {
        AgentRoleBuilderWithChunkHandler {
            inner: self,
            chunk_handler: Box::new(handler),
        }
    }
}

impl<T> McpServerBuilder<T> {
    /// Set binary path - EXACT syntax: .bin("/path/to/bin")
    pub fn bin(mut self, path: impl Into<String>) -> Self {
        self.bin_path = Some(path.into());
        self
    }
    
    /// Initialize - EXACT syntax: .init("cargo run -- --stdio")
    pub fn init(mut self, command: impl Into<String>) -> AgentRoleBuilder {
        let mut parent = self.parent;
        let new_config = McpServerConfig {
            server_type: std::any::type_name::<T>().to_string(),
            bin_path: self.bin_path,
            init_command: Some(command.into()),
        };
        parent.mcp_servers = match parent.mcp_servers {
            Some(mut servers) => {
                servers.push(new_config);
                Some(servers)
            }
            None => Some(ZeroOneOrMany::one(new_config))
        };
        parent
    }
}

/// Builder with chunk handler - has access to terminal methods
pub struct AgentRoleBuilderWithChunkHandler {
    inner: AgentRoleBuilder,
    chunk_handler: Box<dyn Fn(Result<ChatMessageChunk, String>) -> Result<ChatMessageChunk, String> + Send + Sync>,
}

impl AgentRoleBuilderWithChunkHandler {
    /// Convert to agent - EXACT syntax: .into_agent()
    pub fn into_agent(self) -> AgentWithHistory {
        AgentWithHistory {
            inner: self.inner,
            chunk_handler: self.chunk_handler,
            conversation_history: None,
        }
    }
}

/// Agent with conversation history
pub struct AgentWithHistory {
    inner: AgentRoleBuilder,
    chunk_handler: Box<dyn Fn(Result<ChatMessageChunk, String>) -> Result<ChatMessageChunk, String> + Send + Sync>,
    conversation_history: Option<ZeroOneOrMany<(MessageRole, String)>>,
}

impl AgentWithHistory {
    /// Set conversation history - EXACT syntax: .conversation_history(MessageRole::User => "...", ...)
    pub fn conversation_history(mut self, history: impl ConversationHistoryArgs) -> Self {
        self.conversation_history = history.into_history();
        self
    }
    
    /// Start chat - EXACT syntax: .chat("Hello")?
    pub fn chat(self, message: impl Into<String>) -> Result<AsyncStream<ChatMessageChunk>, String> {
        let message = message.into();
        let handler = self.chunk_handler;
        
        // Create channel for streaming chunks
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn task to handle chat
        tokio::spawn(async move {
            // Send conversation history first
            if let Some(history) = self.conversation_history {
                for (role, content) in history.into_iter() {
                let chunk = ChatMessageChunk::new(content, role);
                match handler(Ok(chunk.clone())) {
                    Ok(processed_chunk) => {
                        let _ = tx.send(processed_chunk);
                    }
                    Err(_) => {
                        // Handler rejected this chunk, skip it
                    }
                }
                }
            }
            
            // Send the new user message
            let user_chunk = ChatMessageChunk::new(message.clone(), MessageRole::User);
            match handler(Ok(user_chunk)) {
                Ok(processed_chunk) => {
                    let _ = tx.send(processed_chunk);
                }
                Err(_) => {}
            }
            
            // TODO: Actual implementation will delegate to completion provider
            let response_chunk = ChatMessageChunk::new(
                "Response placeholder", 
                MessageRole::Assistant
            );
            match handler(Ok(response_chunk)) {
                Ok(processed_chunk) => {
                    let _ = tx.send(processed_chunk);
                }
                Err(_) => {}
            }
        });
        
        Ok(AsyncStream::new(rx))
    }
}

/// Trait for context arguments
pub trait ContextArgs {
    fn add_to(self, contexts: &mut Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>);
}

/// Trait for tool arguments
pub trait ToolArgs {
    fn add_to(self, tools: &mut Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>);
}

/// Trait for conversation history arguments
pub trait ConversationHistoryArgs {
    fn into_history(self) -> Option<ZeroOneOrMany<(MessageRole, String)>>;
}

// Implement ContextArgs for tuples to support multiple arguments
// Note: We can't have a blanket impl for T1 because it conflicts with tuple impls
// Users must use tuples for multiple arguments

// Implement for single Context<T> items
impl<T> ContextArgs for crate::domain::context::Context<T> 
where 
    T: Send + Sync + 'static
{
    fn add_to(self, contexts: &mut Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>) {
        match contexts {
            Some(ref mut list) => list.push(Box::new(self)),
            None => *contexts = Some(ZeroOneOrMany::one(Box::new(self))),
        }
    }
}

impl<T1, T2> ContextArgs for (T1, T2)
where 
    T1: std::any::Any + Send + Sync + 'static,
    T2: std::any::Any + Send + Sync + 'static
{
    fn add_to(self, contexts: &mut Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>) {
        match contexts {
            Some(ref mut list) => {
                list.push(Box::new(self.0));
                list.push(Box::new(self.1));
            }
            None => {
                match ZeroOneOrMany::many(vec![Box::new(self.0), Box::new(self.1)]) {
                    Ok(ctx) => *contexts = Some(ctx),
                    Err(_) => {
                        log::error!("Failed to create context list for 2-tuple");
                        *contexts = Some(ZeroOneOrMany::none());
                    }
                }
            }
        }
    }
}

impl<T1, T2, T3> ContextArgs for (T1, T2, T3)
where 
    T1: std::any::Any + Send + Sync + 'static,
    T2: std::any::Any + Send + Sync + 'static,
    T3: std::any::Any + Send + Sync + 'static
{
    fn add_to(self, contexts: &mut Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>) {
        match contexts {
            Some(ref mut list) => {
                list.push(Box::new(self.0));
                list.push(Box::new(self.1));
                list.push(Box::new(self.2));
            }
            None => {
                match ZeroOneOrMany::many(vec![Box::new(self.0), Box::new(self.1), Box::new(self.2)]) {
                    Ok(ctx) => *contexts = Some(ctx),
                    Err(_) => {
                        log::error!("Failed to create context list for 3-tuple");
                        *contexts = Some(ZeroOneOrMany::none());
                    }
                }
            }
        }
    }
}

impl<T1, T2, T3, T4> ContextArgs for (T1, T2, T3, T4)
where 
    T1: std::any::Any + Send + Sync + 'static,
    T2: std::any::Any + Send + Sync + 'static,
    T3: std::any::Any + Send + Sync + 'static,
    T4: std::any::Any + Send + Sync + 'static
{
    fn add_to(self, contexts: &mut Option<ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>>) {
        match contexts {
            Some(ref mut list) => {
                list.push(Box::new(self.0));
                list.push(Box::new(self.1));
                list.push(Box::new(self.2));
                list.push(Box::new(self.3));
            }
            None => {
                match ZeroOneOrMany::many(vec![Box::new(self.0), Box::new(self.1), Box::new(self.2), Box::new(self.3)]) {
                    Ok(ctx) => *contexts = Some(ctx),
                    Err(_) => {
                        log::error!("Failed to create context list for 4-tuple");
                        *contexts = Some(ZeroOneOrMany::none());
                    }
                }
            }
        }
    }
}

// Implement ToolArgs for tuples
// Implement for single Tool<T> items
impl<T> ToolArgs for crate::domain::tool_v2::Tool<T> 
where 
    T: Send + Sync + 'static
{
    fn add_to(self, tools: &mut ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>) {
        *tools = tools.clone().push(Box::new(self));
    }
}

// Implement for NamedTool
impl ToolArgs for crate::domain::tool_v2::NamedTool {
    fn add_to(self, tools: &mut ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>) {
        *tools = tools.clone().push(Box::new(self));
    }
}

impl<T1, T2> ToolArgs for (T1, T2)
where 
    T1: std::any::Any + Send + Sync + 'static,
    T2: std::any::Any + Send + Sync + 'static
{
    fn add_to(self, tools: &mut ZeroOneOrMany<Box<dyn std::any::Any + Send + Sync>>) {
        *tools = tools.clone().push(Box::new(self.0)).push(Box::new(self.1));
    }
}

// Support for conversation history with => syntax
impl ConversationHistoryArgs for ZeroOneOrMany<(MessageRole, String)> {
    fn into_history(self) -> ZeroOneOrMany<(MessageRole, String)> {
        self
    }
}

// Macro to support => syntax for conversation history
#[macro_export]
macro_rules! conversation_history {
    ($($role:path => $msg:expr),* $(,)?) => {{
        let mut history = $crate::ZeroOneOrMany::None;
        $(
            history = history.push(($role, $msg.to_string()));
        )*
        history
    }};
}