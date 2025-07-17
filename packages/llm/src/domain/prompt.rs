use serde::{Deserialize, Serialize};
use crate::async_task::AsyncTask;
use crate::domain::{Message, MessageRole};
use crate::domain::completion::CompletionModel;
use crate::domain::agent::Agent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub content: String,
    #[serde(default = "default_role")]
    pub role: MessageRole,
}

fn default_role() -> MessageRole {
    MessageRole::User
}

impl Prompt {
    pub fn new(content: impl Into<String>) -> Self {
        Prompt {
            content: content.into(),
            role: MessageRole::User,
        }
    }
    
    pub fn content(&self) -> &str {
        &self.content
    }
}

pub struct PromptBuilder {
    content: String,
}

impl Prompt {
    // Semantic entry point
    pub fn ask(content: impl Into<String>) -> PromptBuilder {
        PromptBuilder {
            content: content.into(),
        }
    }
}

impl Into<String> for Prompt {
    fn into(self) -> String {
        self.content
    }
}

impl Into<Prompt> for PromptBuilder {
    fn into(self) -> Prompt {
        Prompt::new(self.content)
    }
}