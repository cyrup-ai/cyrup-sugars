//! Chunk types for streaming operations
//! 
//! These types represent partial data that flows through AsyncStream<T>
//! and are designed to work with the NotResult constraint.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Chunk of document content for streaming file operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    /// Optional path to the source file
    pub path: Option<PathBuf>,
    
    /// The content of this chunk
    pub content: String,
    
    /// Byte range in the original file
    pub byte_range: Option<(usize, usize)>,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Image format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageFormat {
    PNG,
    JPEG,
    GIF,
    WebP,
    BMP,
    TIFF,
}

/// Chunk of image data for streaming image operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageChunk {
    /// Raw image data
    pub data: Vec<u8>,
    
    /// Image format
    pub format: ImageFormat,
    
    /// Optional dimensions (width, height)
    pub dimensions: Option<(u32, u32)>,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Audio format types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AudioFormat {
    MP3,
    WAV,
    FLAC,
    OGG,
    M4A,
    OPUS,
}

/// Chunk of audio/voice data for streaming audio operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceChunk {
    /// Raw audio data
    pub audio_data: Vec<u8>,
    
    /// Audio format
    pub format: AudioFormat,
    
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Chunk of chat message for streaming responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageChunk {
    /// Partial message content
    pub content: String,
    
    /// Role of the message sender
    pub role: crate::domain::message::MessageRole,
    
    /// Whether this is the final chunk
    pub is_final: bool,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Reason why a completion finished
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// Chunk of completion text for streaming completions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChunk {
    /// The text content
    pub text: String,
    
    /// Reason for finishing (if this is the last chunk)
    pub finish_reason: Option<FinishReason>,
    
    /// Token usage information (if available)
    pub usage: Option<Usage>,
}

/// Chunk of embedding data for streaming embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingChunk {
    /// The embedding vector
    pub embeddings: Vec<f32>,
    
    /// Index in the batch
    pub index: usize,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Chunk of transcribed text from speech-to-text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionChunk {
    /// The transcribed text for this chunk
    pub text: String,
    
    /// Confidence score (0.0 to 1.0)
    pub confidence: Option<f32>,
    
    /// Start time in milliseconds
    pub start_time_ms: Option<u64>,
    
    /// End time in milliseconds  
    pub end_time_ms: Option<u64>,
    
    /// Whether this is the final chunk
    pub is_final: bool,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

/// Chunk of synthesized speech for text-to-speech
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechChunk {
    /// Raw audio data
    pub audio_data: Vec<u8>,
    
    /// Audio format
    pub format: AudioFormat,
    
    /// Duration in milliseconds
    pub duration_ms: Option<u64>,
    
    /// Sample rate in Hz
    pub sample_rate: Option<u32>,
    
    /// Whether this is the final chunk
    pub is_final: bool,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, Value>,
}

// Convenience constructors
impl DocumentChunk {
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            path: None,
            content: content.into(),
            byte_range: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_path(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self
    }
    
    pub fn with_range(mut self, start: usize, end: usize) -> Self {
        self.byte_range = Some((start, end));
        self
    }
    
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

impl ChatMessageChunk {
    pub fn new(content: impl Into<String>, role: crate::domain::message::MessageRole) -> Self {
        Self {
            content: content.into(),
            role,
            is_final: false,
            metadata: HashMap::new(),
        }
    }
    
    pub fn final_chunk(mut self) -> Self {
        self.is_final = true;
        self
    }
}

impl CompletionChunk {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            finish_reason: None,
            usage: None,
        }
    }
    
    pub fn finished(mut self, reason: FinishReason) -> Self {
        self.finish_reason = Some(reason);
        self
    }
    
    pub fn with_usage(mut self, usage: Usage) -> Self {
        self.usage = Some(usage);
        self
    }
}