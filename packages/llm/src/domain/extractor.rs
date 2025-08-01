use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use std::marker::PhantomData;
use crate::async_task::AsyncTask;
use crate::domain::{CompletionRequest, Message};
use crate::domain::completion::CompletionModel;
use crate::domain::agent::Agent;
use crate::sugars::FutureExt;

pub struct Extractor<T: DeserializeOwned> {
    pub agent: Agent,
    pub _t: PhantomData<T>,
}

pub struct ExtractorBuilder<T: DeserializeOwned, M: CompletionModel> {
    model: M,
    system_prompt: Option<String>,
    _t: PhantomData<T>,
}

pub struct ExtractorBuilderWithHandler<T: DeserializeOwned, M: CompletionModel> {
    model: M,
    system_prompt: Option<String>,
    error_handler: Box<dyn Fn(String) + Send + Sync>,
    _t: PhantomData<T>,
}

impl<T: DeserializeOwned + Send + 'static + crate::async_task::NotResult> Extractor<T> {
    // Semantic entry point
    pub fn extract_with<M: CompletionModel>(model: M) -> ExtractorBuilder<T, M> {
        ExtractorBuilder {
            model,
            system_prompt: None,
            _t: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Send + 'static + crate::async_task::NotResult, M: CompletionModel> ExtractorBuilder<T, M> {
    pub fn system_prompt(mut self, system_prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(system_prompt.into());
        self
    }
    
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.system_prompt = Some(instructions.into());
        self
    }
    
    // Error handling - required before terminal methods
    pub fn on_error<F>(self, handler: F) -> ExtractorBuilderWithHandler<T, M>
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        ExtractorBuilderWithHandler {
            model: self.model,
            system_prompt: self.system_prompt,
            error_handler: Box::new(handler),
            _t: PhantomData,
        }
    }
}

impl<T: DeserializeOwned + Send + 'static + crate::async_task::NotResult, M: CompletionModel> ExtractorBuilderWithHandler<T, M> {
    // Terminal method - extracts from text
    pub fn from_text(self, text: impl Into<String>) -> AsyncTask<T> {
        let system_prompt = self.system_prompt.unwrap_or_else(|| {
            format!("Extract structured data in JSON format matching the schema for type {}", 
                std::any::type_name::<T>())
        });
            
        // Use prompt() method and collect the stream
        use crate::domain::prompt::Prompt;
        let prompt = Prompt::new(format!("{}\n\n{}", system_prompt, text.into()));
        
        self.model.prompt(prompt).collect_async().map(|chunks| {
            // Combine chunks into full response
            let response = chunks.into_iter()
                .map(|chunk| chunk.text)
                .collect::<String>();
            // Parse JSON response into T
            serde_json::from_str::<T>(&response).map_err(|e| {
                log::error!("Failed to parse JSON response: {}", e);
                e
            })?
        })
    }
    
    // Terminal method with handler
    pub fn on_extraction<F, U>(self, text: impl Into<String>, handler: F) -> AsyncTask<U>
    where
        F: FnOnce(Result<T, String>) -> U + Send + 'static,
        U: Send + 'static + crate::async_task::NotResult,
    {
        self.from_text(text).map(move |extracted| {
            handler(Ok(extracted))
        })
    }
}