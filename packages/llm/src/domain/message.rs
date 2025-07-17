use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use crate::ZeroOneOrMany;

// Message role for context building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

// Core message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
    pub chunk: Option<MessageChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageChunk {
    pub index: usize,
    pub total: Option<usize>,
    pub content: String,
}


impl ZeroOneOrMany<UserContent> {
    /// Extract text content from user content, concatenating multiple items
    pub fn as_text(&self) -> String {
        self.iter()
            .map(|c| c.as_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl ZeroOneOrMany<AssistantContent> {
    /// Extract text content from assistant content, concatenating multiple items
    pub fn as_text(&self) -> String {
        self.iter()
            .map(|c| c.as_text())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl UserContent {
    /// Extract text representation of user content
    pub fn as_text(&self) -> String {
        match self {
            UserContent::Text(text) => text.clone(),
            UserContent::Image(_) => "[Image]".to_string(),
            UserContent::Audio(_) => "[Audio]".to_string(),
            UserContent::Document(doc) => doc.content(),
        }
    }
}

impl AssistantContent {
    /// Extract text representation of assistant content
    pub fn as_text(&self) -> String {
        match self {
            AssistantContent::Text(text) => text.clone(),
            AssistantContent::ToolCall(call) => {
                format!("Tool: {} ({})", call.name, call.arguments)
            },
            AssistantContent::ToolResult(result) => {
                format!("Result: {}", result.result)
            },
        }
    }
}

/// Conversation trait for message conversations
pub trait Conversation {
    /// Convert the conversation to a text representation
    fn as_text(&self) -> String;
}

/// Content trait for message content types
pub trait Content: Serialize {
    /// Convert content to JSON string representation
    fn to_content_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

/// Conversation wrapper around HashMap
#[derive(Debug, Clone)]
pub struct ConversationMap(HashMap<MessageRole, Message>);

impl std::ops::Deref for ConversationMap {
    type Target = HashMap<MessageRole, Message>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HashMap<MessageRole, Message>> for ConversationMap {
    fn from(map: HashMap<MessageRole, Message>) -> Self {
        ConversationMap(map)
    }
}

impl Conversation for ConversationMap {
    fn as_text(&self) -> String {
        let mut text = String::new();
        for (role, message) in self.0.iter() {
            match role {
                MessageRole::User => {
                    text.push_str("User: ");
                    text.push_str(&message.content);
                },
                MessageRole::Assistant => {
                    text.push_str("Assistant: ");
                    text.push_str(&message.content);
                },
                MessageRole::System => {
                    text.push_str("System: ");
                    text.push_str(&message.content);
                }
            }
            text.push('\n');
        }
        text
    }
}

impl Conversation for HashMap<MessageRole, Message> {
    fn as_text(&self) -> String {
        let mut text = String::new();
        for (role, message) in self.iter() {
            match role {
                MessageRole::User => {
                    text.push_str("User: ");
                    text.push_str(&message.content);
                },
                MessageRole::Assistant => {
                    text.push_str("Assistant: ");
                    text.push_str(&message.content);
                },
                MessageRole::System => {
                    text.push_str("System: ");
                    text.push_str(&message.content);
                }
            }
            text.push('\n');
        }
        text
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum UserContent {
    Text(String),
    Image(crate::domain::Image),
    Audio(crate::domain::Audio),
    Document(crate::domain::Document),
}

impl Content for UserContent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AssistantContent {
    Text(String),
    ToolCall(ToolCall),
    ToolResult(ToolResult),
}

impl Content for AssistantContent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub tool_call_id: String,
    pub result: Value,
    pub error: Option<String>,
}

// Direct factory methods - no new(), no build()
impl Message {
    pub fn user(content: impl Into<String>) -> Self {
        Message {
            role: MessageRole::User,
            content: content.into(),
            chunk: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Message {
            role: MessageRole::Assistant,
            content: content.into(),
            chunk: None,
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Message {
            role: MessageRole::System,
            content: content.into(),
            chunk: None,
        }
    }
}

// Message builder traits
pub trait MessageBuilder {
    type Content: Content;
    fn add_content(self, content: Self::Content) -> Self;
    fn build(self) -> Message;
}

pub trait UserMessageBuilderTrait: MessageBuilder<Content = UserContent> {
    fn text(self, text: impl Into<String>) -> Self;
    fn image(self, image: crate::domain::Image) -> Self;
    fn audio(self, audio: crate::domain::Audio) -> Self;
    fn document(self, document: crate::domain::Document) -> Self;
    fn say(self) -> Message;
}

pub trait AssistantMessageBuilderTrait: MessageBuilder<Content = AssistantContent> {
    fn text(self, text: impl Into<String>) -> Self;
    fn tool_call(self, id: impl Into<String>, name: impl Into<String>, arguments: Value) -> Self;
    fn tool_result(self, tool_call_id: impl Into<String>, result: Value) -> Self;
    fn tool_error(self, tool_call_id: impl Into<String>, error: impl Into<String>) -> Self;
    fn respond(self) -> Message;
}

// Trait for creating message builders
pub trait MessageFactory {
    fn user_message() -> impl UserMessageBuilderTrait;
    fn assistant_message() -> impl AssistantMessageBuilderTrait;
}

// Now let's also define a content container trait
pub trait ContentContainer: Content {
    type Item: Content;
    fn items(&self) -> &[Self::Item];
}

// Implementation for ZeroOneOrMany
impl<T: Content + Clone> Content for ZeroOneOrMany<T> {}