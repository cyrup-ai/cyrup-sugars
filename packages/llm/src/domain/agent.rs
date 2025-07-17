use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::async_task::AsyncTask;
use crate::async_task::AsyncStream;
use crate::domain::{Document, VectorStoreIndexDyn, ToolSet, CompletionRequest, Message, MessageRole, Conversation};
use crate::domain::chunk::{ChatMessageChunk, CompletionChunk};
use crate::domain::prompt::Prompt;
use crate::domain::completion::CompletionRequestBuilder;
use crate::domain::prompt::PromptBuilder;
use crate::{ZeroOneOrMany, McpTool};
use crate::sugars::{ByteSize, ByteSizeExt};
use crate::memory::Memory;

pub struct Agent {
    pub model: Box<dyn std::any::Any + Send + Sync>,
    pub system_prompt: String,
    pub context: ZeroOneOrMany<Document>,
    pub tools: ZeroOneOrMany<McpTool>,
    pub memory: Option<Memory>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
    pub additional_params: Option<Value>,
}

pub struct AgentBuilder {
    model: Box<dyn std::any::Any + Send + Sync>,
    system_prompt: String,
    context: Option<ZeroOneOrMany<Document>>,
    tools: Option<ZeroOneOrMany<McpTool>>,
    memory: Option<Memory>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    additional_params: Option<Value>,
}

pub struct AgentBuilderWithHandler {
    model: Box<dyn std::any::Any + Send + Sync>,
    system_prompt: String,
    context: Option<ZeroOneOrMany<Document>>,
    tools: Option<ZeroOneOrMany<McpTool>>,
    memory: Option<Memory>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    additional_params: Option<Value>,
    error_handler: Box<dyn Fn(String) + Send + Sync>,
}

impl Agent {
    // Semantic entry point
    pub fn with_model(model: impl std::any::Any + Send + Sync + 'static) -> AgentBuilder {
        AgentBuilder {
            model: Box::new(model),
            system_prompt: String::new(),
            context: None,
            tools: None,
            memory: None,
            temperature: None,
            max_tokens: None,
            additional_params: None,
        }
    }
}

impl AgentBuilder {
    pub fn system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = system_prompt.into();
        self
    }
    
    pub fn context<F, C>(mut self, f: F) -> Self 
    where 
        F: FnOnce() -> C,
        C: crate::domain::Conversation
    {
        let conversation = f();
        let text = conversation.as_text();
        let doc = Document::from_text(text).load();
        
        match self.context {
            Some(mut existing) => {
                existing.push(doc);
                self.context = Some(existing);
            }
            None => {
                self.context = Some(ZeroOneOrMany::one(doc));
            }
        }
        self
    }
    
    pub fn add_context(mut self, document: Document) -> Self {
        match self.context {
            Some(mut existing) => {
                existing.push(document);
                self.context = Some(existing);
            }
            None => {
                self.context = Some(ZeroOneOrMany::one(document));
            }
        }
        self
    }
    
    /// Convenience method to add context from a simple text string
    pub fn context_text(mut self, text: impl Into<String>) -> Self {
        let doc = Document::from_text(text.into()).load();
        match self.context {
            Some(mut existing) => {
                existing.push(doc);
                self.context = Some(existing);
            }
            None => {
                self.context = Some(ZeroOneOrMany::one(doc));
            }
        }
        self
    }
    
    pub fn tool(mut self, tool: impl Into<McpTool>) -> Self {
        let tool = tool.into();
        match self.tools {
            Some(mut existing) => {
                existing.push(tool);
                self.tools = Some(existing);
            }
            None => {
                self.tools = Some(ZeroOneOrMany::one(tool));
            }
        }
        self
    }
    
    
    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }
    
    pub fn max_tokens(mut self, max: u64) -> Self {
        self.max_tokens = Some(max);
        self
    }
    
    pub fn additional_params(mut self, params: Value) -> Self {
        self.additional_params = Some(params);
        self
    }
    
    pub fn memory(mut self, memory: Memory) -> Self {
        self.memory = Some(memory);
        self
    }
    
    pub fn with_memory<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(Memory) -> Memory
    {
        let memory = Memory::new();
        self.memory = Some(f(memory));
        self
    }
    
    pub fn metadata<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce() -> hashbrown::HashMap<String, Value>
    {
        let metadata = f();
        let json_metadata: serde_json::Map<String, Value> = metadata
            .into_iter()
            .collect();
        self.additional_params = Some(Value::Object(json_metadata));
        self
    }
    
    // Error handling - required before terminal methods
    pub fn on_error<F>(self, handler: F) -> AgentBuilderWithHandler
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        AgentBuilderWithHandler {
            model: self.model,
            system_prompt: self.system_prompt,
            context: self.context,
            tools: self.tools,
            memory: self.memory,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            additional_params: self.additional_params,
            error_handler: Box::new(handler),
        }
    }
}

// Implementation for AgentBuilderWithHandler - has terminal methods
impl AgentBuilderWithHandler {
    // Terminal method - creates agent
    pub fn agent(self) -> Agent {
        let default_tool = McpTool::new(
            "default",
            "Default tool",
            serde_json::json!({}),
        );
        
        Agent {
            model: self.model,
            system_prompt: self.system_prompt,
            context: self.context.unwrap_or_else(|| {
                ZeroOneOrMany::one(Document::from_text("").load())
            }),
            tools: self.tools.unwrap_or_else(|| ZeroOneOrMany::one(default_tool)),
            memory: self.memory,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            additional_params: self.additional_params,
        }
    }
    
    // Terminal method - chat interaction  
    pub fn chat(self, message: impl Into<String>) -> AsyncStream<ChatMessageChunk> {
        let agent = self.agent();
        let message = message.into();
        
        // Create channel for streaming chunks
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn task to handle chat with tool looping
        tokio::spawn(async move {
            // Initial user message
            let user_chunk = ChatMessageChunk::new(message.clone(), MessageRole::User);
            let _ = tx.send(user_chunk);
            
            // TODO: Implement actual agent chat logic with tool calling loop
            // For now, just send a simple response
            let response_chunk = ChatMessageChunk::new(
                "I'm an agent that will handle tool calling internally", 
                MessageRole::Assistant
            );
            let _ = tx.send(response_chunk);
        });
        
        AsyncStream::new(rx)
    }
    
    // Terminal method - stream completion
    pub fn stream_completion(self, prompt: impl Into<String>) -> AsyncStream<CompletionChunk> {
        let agent = self.agent();
        let request = CompletionRequest::prompt(prompt)
            .temperature(agent.temperature.unwrap_or(0.7))
            .max_tokens(agent.max_tokens.unwrap_or(1000))
            .additional_params(agent.additional_params.clone().unwrap_or_default())
            .on_error(|e| eprintln!("Completion error: {}", e))
            .request();
            
        // TODO: Implement actual completion streaming
        // For now, return empty stream
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        AsyncStream::new(rx)
    }
    
    // Terminal method - chat with chunk handler
    pub fn on_chunk<F>(self, message: impl Into<String>, handler: F) -> AsyncStream<ChatMessageChunk>
    where
        F: Fn(ChatMessageChunk) + Send + Sync + 'static,
    {
        // Default chunk size of 512 bytes
        self.on_chunk_with_size(message, handler, 512.bytes())
    }
    
    // Terminal method - chat with chunk handler and custom size
    pub fn on_chunk_with_size<F>(
        self,
        message: impl Into<String>,
        handler: F,
        chunk_size: ByteSize,
    ) -> AsyncStream<ChatMessageChunk>
    where
        F: Fn(ChatMessageChunk) + Send + Sync + 'static,
    {
        let message = message.into();
            
        // Create channel for streaming chunks
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn task to handle chat with tool looping
        let agent = self.agent();
        tokio::spawn(async move {
            // Initial user message
            let user_chunk = ChatMessageChunk::new(message.clone(), MessageRole::User);
            let _ = tx.send(user_chunk.clone());
            handler(user_chunk);
            
            // TODO: Implement actual agent chat logic with tool calling loop
            // For now, just send a simple response
            let response_chunk = ChatMessageChunk::new(
                "I'm an agent that will handle tool calling internally", 
                MessageRole::Assistant
            );
            let _ = tx.send(response_chunk.clone());
            handler(response_chunk);
        });
        
        AsyncStream::new(rx)
    }
    
    // Terminal method - create a completion request
    pub fn completion(self, prompt: impl Into<String>) -> CompletionRequestBuilder {
        let agent = self.agent();
        CompletionRequest::prompt(prompt)
            .temperature(agent.temperature.unwrap_or(0.7))
            .max_tokens(agent.max_tokens.unwrap_or(1000))
            .additional_params(agent.additional_params.clone().unwrap_or_default())
    }
    
    // Terminal method - start a conversation
    pub fn conversation(self) -> ConversationBuilder {
        let agent = self.agent();
        ConversationBuilder::with_agent(agent)
    }
    
    // Terminal method with handler
    pub fn on_response<F>(self, message: impl Into<String>, handler: F) -> AsyncTask<String>
    where
        F: FnOnce(Result<String, String>) -> String + Send + 'static,
    {
        let agent = self.agent();
        // TODO: Implement actual completion with the model
        // For now, return a placeholder response
        AsyncTask::spawn(move || {
            let response = "Placeholder response".to_string();
            handler(Ok(response))
        })
    }
}

// Conversation builder for agent context
pub struct ConversationBuilder {
    agent: Agent,
    messages: Vec<Message>,
}

impl ConversationBuilder {
    pub fn with_agent(agent: Agent) -> Self {
        Self {
            agent,
            messages: Vec::new(),
        }
    }
    
    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::system(content));
        self
    }
    
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::user(content));
        self
    }
    
    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::assistant(content));
        self
    }
    
    pub fn message(mut self, message: Message) -> Self {
        self.messages.push(message);
        self
    }
    
    // Terminal method - returns chat history
    pub fn history(self) -> Vec<Message> {
        self.messages
    }
    
    // Terminal method - starts conversation
    pub fn converse(self) -> AsyncStream<ChatMessageChunk> {
        // For conversation, we need to convert messages to a prompt
        // Taking the last user message as the prompt
        let last_user_message = self.messages.iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
            .map(|m| m.content.clone())
            .unwrap_or_default();
            
        // Create channel for streaming chunks  
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        // Spawn task to handle conversation chat
        tokio::spawn(async move {
            // TODO: Implement actual conversation handling with message history
            // For now, just send a simple response
            let response_chunk = ChatMessageChunk::new(
                "Conversation handling with history", 
                MessageRole::Assistant
            );
            let _ = tx.send(response_chunk);
        });
        
        AsyncStream::new(rx)
    }
}