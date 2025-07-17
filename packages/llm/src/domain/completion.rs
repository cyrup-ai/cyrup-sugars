use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::async_task::AsyncTask;
use crate::async_task::AsyncStream;
use crate::domain::chunk::{ChatMessageChunk, CompletionChunk};
use crate::domain::prompt::Prompt;

/// Core trait for completion models
pub trait CompletionModel: Send + Sync + Clone {
    /// Generate completion from prompt
    fn prompt(&self, prompt: Prompt) -> AsyncStream<CompletionChunk>;
}

pub trait CompletionBackend {
    fn submit_completion(
        &self,
        prompt: &str,
        tools: &[String],
    ) -> crate::async_task::AsyncTask<String>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub system_prompt: String,
    pub chat_history: Vec<crate::domain::Message>,
    pub documents: Vec<crate::domain::Document>,
    pub tools: Vec<ToolDefinition>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
    pub chunk_size: Option<usize>,
    pub additional_params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

pub struct CompletionRequestBuilder {
    system_prompt: Option<String>,
    chat_history: Vec<crate::domain::Message>,
    documents: Vec<crate::domain::Document>,
    tools: Vec<ToolDefinition>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    chunk_size: Option<usize>,
    additional_params: Option<Value>,
}

pub struct CompletionRequestBuilderWithHandler {
    system_prompt: Option<String>,
    chat_history: Vec<crate::domain::Message>,
    documents: Vec<crate::domain::Document>,
    tools: Vec<ToolDefinition>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
    chunk_size: Option<usize>,
    additional_params: Option<Value>,
    error_handler: Box<dyn Fn(String) + Send + Sync>,
}

impl CompletionRequest {
    // Semantic entry point
    pub fn prompt(system_prompt: impl Into<String>) -> CompletionRequestBuilder {
        CompletionRequestBuilder {
            system_prompt: Some(system_prompt.into()),
            chat_history: Vec::new(),
            documents: Vec::new(),
            tools: Vec::new(),
            temperature: None,
            max_tokens: None,
            chunk_size: None,
            additional_params: None,
        }
    }

}

impl CompletionRequestBuilder {
    pub fn system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }

    pub fn chat_history(mut self, history: Vec<crate::domain::Message>) -> Self {
        self.chat_history = history;
        self
    }

    pub fn add_message(mut self, message: crate::domain::Message) -> Self {
        self.chat_history.push(message);
        self
    }

    // Message creation in context
    pub fn user(mut self, content: impl Into<String>) -> Self {
        self.chat_history.push(crate::domain::Message::user(content));
        self
    }

    pub fn assistant(mut self, content: impl Into<String>) -> Self {
        self.chat_history.push(crate::domain::Message::assistant(content));
        self
    }

    pub fn system(mut self, content: impl Into<String>) -> Self {
        self.chat_history.push(crate::domain::Message::system(content));
        self
    }

    pub fn documents(mut self, documents: Vec<crate::domain::Document>) -> Self {
        self.documents = documents;
        self
    }

    pub fn add_document(mut self, document: crate::domain::Document) -> Self {
        self.documents.push(document);
        self
    }

    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    pub fn add_tool(mut self, name: impl Into<String>, description: impl Into<String>, parameters: Value) -> Self {
        self.tools.push(ToolDefinition {
            name: name.into(),
            description: description.into(),
            parameters,
        });
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

    pub fn chunk_size(mut self, size: usize) -> Self {
        self.chunk_size = Some(size);
        self
    }

    pub fn additional_params(mut self, params: Value) -> Self {
        self.additional_params = Some(params);
        self
    }

    pub fn params<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<String, Value>
    {
        let params = f();
        let json_params: serde_json::Map<String, Value> = params
            .into_iter()
            .collect();
        self.additional_params = Some(Value::Object(json_params));
        self
    }

    // Error handling - required before terminal methods
    pub fn on_error<F>(self, handler: F) -> CompletionRequestBuilderWithHandler
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        CompletionRequestBuilderWithHandler {
            system_prompt: self.system_prompt,
            chat_history: self.chat_history,
            documents: self.documents,
            tools: self.tools,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            chunk_size: self.chunk_size,
            additional_params: self.additional_params,
            error_handler: Box::new(handler),
        }
    }
}

impl CompletionRequestBuilderWithHandler {
    // Terminal method - returns CompletionRequest
    pub fn request(self) -> CompletionRequest {
        CompletionRequest {
            system_prompt: self.system_prompt.unwrap_or_default(),
            chat_history: self.chat_history,
            documents: self.documents,
            tools: self.tools,
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            chunk_size: self.chunk_size,
            additional_params: self.additional_params,
        }
    }

    // Terminal method - submits request and returns stream
    pub fn complete<F>(self, handler: F) -> AsyncStream<CompletionChunk>
    where
        F: Fn(CompletionRequest) -> AsyncStream<CompletionChunk> + Send + 'static,
    {
        handler(self.request())
    }

    // Terminal method with result handling
    pub fn on_completion<F>(self, f: F) -> AsyncTask<String>
    where
        F: FnOnce(Result<String, String>) -> String + Send + 'static,
    {
        AsyncTask::spawn(move || {
            // Simulate completion
            let result = Ok("Completion response".to_string());
            f(result)
        })
    }
}
