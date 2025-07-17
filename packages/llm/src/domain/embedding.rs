use serde::{Deserialize, Serialize};
use crate::async_task::AsyncTask;
use crate::async_task::AsyncStream;
use crate::domain::chunk::EmbeddingChunk;
use crate::sugars::FutureExt;

/// Core trait for embedding models
pub trait EmbeddingModel: Send + Sync + Clone {
    /// Create embeddings for a single text
    fn embed(&self, text: &str) -> AsyncTask<Vec<f32>>;
    
    /// Create embeddings for multiple texts with streaming
    fn embed_batch(&self, texts: Vec<String>) -> AsyncStream<EmbeddingChunk>;
    
    /// Standard embedding with handler
    fn on_embedding<F>(&self, text: &str, handler: F) -> AsyncTask<Vec<f32>>
    where
        F: FnOnce(Result<Vec<f32>, String>) -> Vec<f32> + Send + 'static,
        Vec<f32>: crate::async_task::NotResult,
    {
        self.embed(text).map(move |embedding| {
            handler(Ok(embedding))
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embedding {
    pub document: String,
    pub vec: Vec<f64>,
}

pub struct EmbeddingBuilder {
    document: String,
    vec: Option<Vec<f64>>,
}

pub struct EmbeddingBuilderWithHandler {
    document: String,
    vec: Option<Vec<f64>>,
    error_handler: Box<dyn Fn(String) + Send + Sync>,
}

impl Embedding {
    // Semantic entry point
    pub fn from_document(document: impl Into<String>) -> EmbeddingBuilder {
        EmbeddingBuilder {
            document: document.into(),
            vec: None,
        }
    }
}

impl EmbeddingBuilder {
    pub fn vec(mut self, vec: Vec<f64>) -> Self {
        self.vec = Some(vec);
        self
    }
    
    pub fn with_dimensions(mut self, dims: usize) -> Self {
        self.vec = Some(vec![0.0; dims]);
        self
    }
    
    // Error handling - required before terminal methods
    pub fn on_error<F>(self, handler: F) -> EmbeddingBuilderWithHandler
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        EmbeddingBuilderWithHandler {
            document: self.document,
            vec: self.vec,
            error_handler: Box::new(handler),
        }
    }
}

impl EmbeddingBuilderWithHandler {
    // Terminal method - returns AsyncTask<Embedding>
    pub fn embed(self) -> AsyncTask<Embedding> {
        AsyncTask::from_value(Embedding {
            document: self.document,
            vec: self.vec.unwrap_or_default(),
        })
    }
    
}