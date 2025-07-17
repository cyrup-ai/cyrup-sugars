use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::{AsyncTask, AsyncStream};
use crate::domain::chunk::ChatMessageChunk;
use crate::domain::message::MessageRole;
use std::future::Future;
use std::pin::Pin;


// Core tool traits
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    fn execute(&self, args: Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>;
}

pub trait ToolEmbedding: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> Value;
    fn execute(&self, args: Value) -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>>;
    fn embedding(&self) -> &[f64];
}

pub enum ToolType {
    Direct(Box<dyn Tool>),
    Embedding(Box<dyn ToolEmbedding>),
}

pub struct ToolSet {
    tools: Vec<ToolType>,
}

pub struct ToolSetBuilder {
    tools: Vec<ToolType>,
}

impl ToolSet {
    // Direct creation
    pub fn from_tools(tools: Vec<ToolType>) -> Self {
        ToolSet { tools }
    }

    // Semantic builder entry
    pub fn tools() -> ToolSetBuilder {
        ToolSetBuilder {
            tools: Vec::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &ToolType> {
        self.tools.iter()
    }

    // Execute tool by name - returns AsyncStream<ChatMessageChunk>
    pub fn execute(&self, name: &str, args: Value) -> AsyncStream<ChatMessageChunk> {
        for tool in &self.tools {
            let tool_name = match tool {
                ToolType::Direct(t) => t.name(),
                ToolType::Embedding(t) => t.name(),
            };

            if tool_name == name {
                // Create a message chunk with tool execution result
                let chunk = ChatMessageChunk {
                    content: format!("Executing tool: {}", name),
                    role: MessageRole::Assistant,
                    is_final: true,
                    metadata: std::collections::HashMap::new(),
                };
                
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                let _ = tx.send(chunk);
                return AsyncStream::new(rx);
            }
        }

        // Tool not found - return error chunk
        let error_chunk = ChatMessageChunk {
            content: format!("Tool '{}' not found", name),
            role: MessageRole::System,
            is_final: true,
            metadata: std::collections::HashMap::new(),
        };
        
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = tx.send(error_chunk);
        AsyncStream::new(rx)
    }
}

impl ToolSetBuilder {
    pub fn add_tool(mut self, tool: ToolType) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn simple<T: Tool + 'static>(mut self, tool: T) -> Self {
        self.tools.push(ToolType::Direct(Box::new(tool)));
        self
    }

    pub fn embedding<T: ToolEmbedding + 'static>(mut self, tool: T) -> Self {
        self.tools.push(ToolType::Embedding(Box::new(tool)));
        self
    }

    // Terminal method
    pub fn register(self) -> ToolSet {
        ToolSet {
            tools: self.tools,
        }
    }

    // Terminal method with execution
    pub fn on_call<F>(self, name: &str, args: Value, handler: F) -> AsyncTask<Value>
    where
        F: FnOnce(Result<Value, String>) -> Value + Send + 'static,
        Value: crate::async_task::NotResult,
    {
        let toolset = self.register();
        let name = name.to_string();
        AsyncTask::spawn(move || {
            // For now, just return error
            // TODO: Implement actual tool execution
            handler(Err(format!("Tool execution not implemented for '{}'", name)))
        })
    }
}
