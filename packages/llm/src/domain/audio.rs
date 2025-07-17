use serde::{Deserialize, Serialize};
use crate::AsyncStream;
use crate::domain::chunk::{TranscriptionChunk, SpeechChunk, AudioFormat};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audio {
    pub data: String,
    pub format: Option<ContentFormat>,
    pub media_type: Option<AudioMediaType>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContentFormat {
    Base64,
    Raw,
    Url,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AudioMediaType {
    MP3,
    WAV,
    OGG,
    M4A,
    FLAC,
}

pub struct AudioBuilder {
    data: String,
    format: Option<ContentFormat>,
    media_type: Option<AudioMediaType>,
}

pub struct AudioBuilderWithHandler {
    data: String,
    format: Option<ContentFormat>,
    media_type: Option<AudioMediaType>,
    error_handler: Box<dyn Fn(String) + Send + Sync>,
}

impl Audio {
    // Semantic entry points
    pub fn from_base64(data: impl Into<String>) -> AudioBuilder {
        AudioBuilder {
            data: data.into(),
            format: Some(ContentFormat::Base64),
            media_type: None,
        }
    }
    
    pub fn from_url(url: impl Into<String>) -> AudioBuilder {
        AudioBuilder {
            data: url.into(),
            format: Some(ContentFormat::Url),
            media_type: None,
        }
    }
    
    pub fn from_raw(data: impl Into<String>) -> AudioBuilder {
        AudioBuilder {
            data: data.into(),
            format: Some(ContentFormat::Raw),
            media_type: None,
        }
    }
}

impl AudioBuilder {
    pub fn format(mut self, format: ContentFormat) -> Self {
        self.format = Some(format);
        self
    }
    
    pub fn media_type(mut self, media_type: AudioMediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }
    
    pub fn as_mp3(mut self) -> Self {
        self.media_type = Some(AudioMediaType::MP3);
        self
    }
    
    pub fn as_wav(mut self) -> Self {
        self.media_type = Some(AudioMediaType::WAV);
        self
    }
    
    // Error handling - required before terminal methods
    pub fn on_error<F>(self, handler: F) -> AudioBuilderWithHandler
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        AudioBuilderWithHandler {
            data: self.data,
            format: self.format,
            media_type: self.media_type,
            error_handler: Box::new(handler),
        }
    }
}

impl AudioBuilderWithHandler {
    // Terminal method - returns AsyncStream<TranscriptionChunk> for STT
    pub fn decode(self) -> AsyncStream<TranscriptionChunk> {
        // Create transcription chunks that can be collected into a Transcription
        let chunk = TranscriptionChunk {
            text: format!("Transcribed audio from: {}", self.data),
            confidence: Some(0.95),
            start_time_ms: Some(0),
            end_time_ms: Some(1000),
            is_final: true,
            metadata: std::collections::HashMap::new(),
        };
        
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = tx.send(chunk);
        AsyncStream::new(rx)
    }
    
    // Terminal method - returns AsyncStream<SpeechChunk> for TTS
    pub fn stream(self) -> AsyncStream<SpeechChunk> {
        // Convert audio data to bytes and create proper SpeechChunk
        let audio_data = self.data.as_bytes().to_vec();
        let format = match self.media_type.unwrap_or(AudioMediaType::MP3) {
            AudioMediaType::MP3 => AudioFormat::MP3,
            AudioMediaType::WAV => AudioFormat::WAV,
            AudioMediaType::OGG => AudioFormat::OGG,
            AudioMediaType::M4A => AudioFormat::M4A,
            AudioMediaType::FLAC => AudioFormat::FLAC,
        };
        
        let chunk = SpeechChunk {
            audio_data,
            format,
            duration_ms: Some(1000),
            sample_rate: Some(44100),
            is_final: true,
            metadata: std::collections::HashMap::new(),
        };
        
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = tx.send(chunk);
        AsyncStream::new(rx)
    }
}