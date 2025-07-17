use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::async_task::AsyncTask;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

#[derive(Debug)]
pub enum VectorStoreError {
    NotFound,
    ConnectionError(String),
    InvalidQuery(String),
}

pub trait VectorStoreIndexDyn: Send + Sync {
    fn top_n(&self, query: &str, n: usize) -> BoxFuture<Result<Vec<(f64, String, Value)>, VectorStoreError>>;
    fn top_n_ids(&self, query: &str, n: usize) -> BoxFuture<Result<Vec<(f64, String)>, VectorStoreError>>;
}

pub struct VectorStoreIndex {
    backend: Box<dyn VectorStoreIndexDyn>,
}

pub struct VectorQueryBuilder<'a> {
    index: &'a VectorStoreIndex,
    query: String,
    n: usize,
}

impl VectorStoreIndex {
    // Direct creation from backend
    pub fn with_backend<B: VectorStoreIndexDyn + 'static>(backend: B) -> Self {
        VectorStoreIndex {
            backend: Box::new(backend),
        }
    }
    
    // Semantic query entry point
    pub fn search(&self, query: impl Into<String>) -> VectorQueryBuilder {
        VectorQueryBuilder {
            index: self,
            query: query.into(),
            n: 10, // default
        }
    }
}

impl<'a> VectorQueryBuilder<'a> {
    pub fn top(mut self, n: usize) -> Self {
        self.n = n;
        self
    }
    
    // Terminal method - returns full results with metadata
    pub fn retrieve(self) -> AsyncTask<Vec<(f64, String, Value)>> {
        let future = self.index.backend.top_n(&self.query, self.n);
        AsyncTask::spawn(move || {
            // This would properly await the future
            vec![]
        })
    }
    
    // Terminal method - returns just IDs
    pub fn retrieve_ids(self) -> AsyncTask<Vec<(f64, String)>> {
        let future = self.index.backend.top_n_ids(&self.query, self.n);
        AsyncTask::spawn(move || {
            // This would properly await the future
            vec![]
        })
    }
    
    // Terminal method with result handler
    pub fn on_results<F, T>(self, handler: F) -> AsyncTask<T>
    where
        F: FnOnce(Vec<(f64, String, Value)>) -> T + Send + 'static,
        T: Send + 'static + crate::async_task::NotResult,
    {
        let future = self.index.backend.top_n(&self.query, self.n);
        AsyncTask::spawn(move || {
            // This would properly await the future and pass to handler
            let result = vec![];
            handler(result)
        })
    }
}