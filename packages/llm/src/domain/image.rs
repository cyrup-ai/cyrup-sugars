use serde::{Deserialize, Serialize};
use crate::{AsyncTask, AsyncStream};
use crate::domain::chunk::ImageChunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub data: String,
    pub format: Option<ContentFormat>,
    pub media_type: Option<ImageMediaType>,
    pub detail: Option<ImageDetail>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContentFormat {
    Base64,
    Url,
    Raw,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ImageMediaType {
    PNG,
    JPEG,
    GIF,
    WEBP,
    SVG,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageDetail {
    Low,
    High,
    Auto,
}

pub struct ImageBuilder {
    data: String,
    format: Option<ContentFormat>,
    media_type: Option<ImageMediaType>,
    detail: Option<ImageDetail>,
}

pub struct ImageBuilderWithHandler {
    data: String,
    format: Option<ContentFormat>,
    media_type: Option<ImageMediaType>,
    detail: Option<ImageDetail>,
    error_handler: Box<dyn Fn(String) + Send + Sync>,
}

impl Image {
    // Semantic entry points
    pub fn from_base64(data: impl Into<String>) -> ImageBuilder {
        ImageBuilder {
            data: data.into(),
            format: Some(ContentFormat::Base64),
            media_type: None,
            detail: None,
        }
    }
    
    pub fn from_url(url: impl Into<String>) -> ImageBuilder {
        ImageBuilder {
            data: url.into(),
            format: Some(ContentFormat::Url),
            media_type: None,
            detail: None,
        }
    }
    
    pub fn from_path(path: impl Into<String>) -> ImageBuilder {
        ImageBuilder {
            data: path.into(),
            format: Some(ContentFormat::Url),
            media_type: None,
            detail: None,
        }
    }
}

impl ImageBuilder {
    pub fn format(mut self, format: ContentFormat) -> Self {
        self.format = Some(format);
        self
    }
    
    pub fn media_type(mut self, media_type: ImageMediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }
    
    pub fn detail(mut self, detail: ImageDetail) -> Self {
        self.detail = Some(detail);
        self
    }
    
    pub fn as_png(mut self) -> Self {
        self.media_type = Some(ImageMediaType::PNG);
        self
    }
    
    pub fn as_jpeg(mut self) -> Self {
        self.media_type = Some(ImageMediaType::JPEG);
        self
    }
    
    pub fn high_detail(mut self) -> Self {
        self.detail = Some(ImageDetail::High);
        self
    }
    
    pub fn low_detail(mut self) -> Self {
        self.detail = Some(ImageDetail::Low);
        self
    }
    
    // Error handling - required before terminal methods
    pub fn on_error<F>(self, handler: F) -> ImageBuilderWithHandler
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        ImageBuilderWithHandler {
            data: self.data,
            format: self.format,
            media_type: self.media_type,
            detail: self.detail,
            error_handler: Box::new(handler),
        }
    }
}

impl ImageBuilderWithHandler {
    // Terminal method - returns AsyncStream<ImageChunk>
    pub fn load(self) -> AsyncStream<ImageChunk> {
        let image = Image {
            data: self.data,
            format: self.format,
            media_type: self.media_type,
            detail: self.detail,
        };
        
        // Convert image data to bytes and create proper ImageChunk
        let data = image.data.as_bytes().to_vec();
        let format = match image.media_type.unwrap_or(ImageMediaType::PNG) {
            ImageMediaType::PNG => crate::domain::chunk::ImageFormat::PNG,
            ImageMediaType::JPEG => crate::domain::chunk::ImageFormat::JPEG,
            ImageMediaType::GIF => crate::domain::chunk::ImageFormat::GIF,
            ImageMediaType::WEBP => crate::domain::chunk::ImageFormat::WebP,
            ImageMediaType::SVG => crate::domain::chunk::ImageFormat::PNG, // fallback
        };
        
        let chunk = ImageChunk {
            data,
            format,
            dimensions: None,
            metadata: std::collections::HashMap::new(),
        };
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = tx.send(chunk);
        AsyncStream::new(rx)
    }
    
    // Terminal method - async load with processing
    pub fn process<F>(self, f: F) -> AsyncStream<ImageChunk>
    where
        F: FnOnce(ImageChunk) -> ImageChunk + Send + 'static,
    {
        // For now, just return the load stream
        // TODO: Implement actual processing
        self.load()
    }
}