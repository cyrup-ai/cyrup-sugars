use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde_json::Value;
use crate::{AsyncTask, AsyncStream};
use crate::domain::chunk::DocumentChunk;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub data: String,
    pub format: Option<ContentFormat>,
    pub media_type: Option<DocumentMediaType>,
    #[serde(flatten)]
    pub additional_props: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContentFormat {
    Base64,
    Text,
    Html,
    Markdown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum DocumentMediaType {
    PDF,
    DOCX,
    TXT,
    RTF,
    ODT,
}

/// Internal enum for builder data sources
enum DocumentBuilderData {
    Immediate(String),
    File(PathBuf),
    Glob(String),
    Url(String),
    GitHub(String),
}

pub struct DocumentBuilder {
    data: DocumentBuilderData,
    format: Option<ContentFormat>,
    media_type: Option<DocumentMediaType>,
    additional_props: HashMap<String, Value>,
    error_handler: Option<Box<dyn FnMut(String) + Send + 'static>>,
}

pub struct DocumentBuilderWithHandler {
    data: DocumentBuilderData,
    format: Option<ContentFormat>,
    media_type: Option<DocumentMediaType>,
    additional_props: HashMap<String, Value>,
    error_handler: Box<dyn FnMut(String) + Send + 'static>,
}

impl Document {
    // Direct semantic entry point - no new()
    pub fn from_data(data: impl Into<String>) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::Immediate(data.into()),
            format: None,
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }

    pub fn from_base64(data: impl Into<String>) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::Immediate(data.into()),
            format: Some(ContentFormat::Base64),
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }

    pub fn from_text(data: impl Into<String>) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::Immediate(data.into()),
            format: Some(ContentFormat::Text),
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }
    
    /// Create a DocumentBuilder from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::File(path.as_ref().to_path_buf()),
            format: None,
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }
    
    /// Create a DocumentBuilder from a glob pattern
    pub fn from_glob(pattern: &str) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::Glob(pattern.to_string()),
            format: None,
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }

    /// Create a DocumentBuilder from a URL
    pub fn from_url(url: &str) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::Url(url.to_string()),
            format: None,
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }
    
    /// Create a DocumentBuilder from a GitHub file path
    /// Format: "owner/repo/path/to/file.txt"
    pub fn from_github(path: &str) -> DocumentBuilder {
        DocumentBuilder {
            data: DocumentBuilderData::GitHub(path.to_string()),
            format: None,
            media_type: None,
            additional_props: HashMap::new(),
            error_handler: None,
        }
    }

    /// Extract the text content from the document
    pub fn content(&self) -> String {
        match self.format {
            Some(ContentFormat::Base64) => "[Base64 Document]".to_string(),
            _ => self.data.clone(),
        }
    }
}

impl DocumentBuilder {
    pub fn format(mut self, format: ContentFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn media_type(mut self, media_type: DocumentMediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.additional_props.insert(key.into(), value);
        self
    }

    pub fn properties<F>(mut self, f: F) -> Self
    where
        F: FnOnce() -> hashbrown::HashMap<String, Value>
    {
        let props = f();
        for (key, value) in props {
            self.additional_props.insert(key, value);
        }
        self
    }
    
    /// Required: Provide error handler to enable async terminal methods
    pub fn on_error<F>(self, error_handler: F) -> DocumentBuilderWithHandler
    where
        F: FnMut(String) + Send + 'static,
    {
        DocumentBuilderWithHandler {
            data: self.data,
            format: self.format,
            media_type: self.media_type,
            additional_props: self.additional_props,
            error_handler: Box::new(error_handler),
        }
    }

    // Sync terminal method - only for immediate data
    pub fn load(self) -> Document {
        match self.data {
            DocumentBuilderData::Immediate(data) => Document {
                data,
                format: self.format,
                media_type: self.media_type,
                additional_props: self.additional_props,
            },
            _ => panic!("load() can only be used with immediate data. Use on_error() and load_async() for file/url/glob operations."),
        }
    }
}

impl DocumentBuilderWithHandler {
    pub fn format(mut self, format: ContentFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn media_type(mut self, media_type: DocumentMediaType) -> Self {
        self.media_type = Some(media_type);
        self
    }

    pub fn property(mut self, key: impl Into<String>, value: Value) -> Self {
        self.additional_props.insert(key.into(), value);
        self
    }

    /// Load document asynchronously
    pub fn load_async(self) -> AsyncTask<Document> {
        match self.data {
            DocumentBuilderData::Immediate(data) => {
                AsyncTask::from_value(Document {
                    data,
                    format: self.format,
                    media_type: self.media_type,
                    additional_props: self.additional_props,
                })
            }
            DocumentBuilderData::File(path) => {
                let format = self.format;
                let media_type = self.media_type;
                let additional_props = self.additional_props;
                let mut error_handler = self.error_handler;
                
                AsyncTask::new(async move {
                    match tokio::fs::read_to_string(&path).await {
                        Ok(data) => Document {
                            data,
                            format,
                            media_type,
                            additional_props,
                        },
                        Err(e) => {
                            error_handler(format!("Failed to read file {:?}: {}", path, e));
                            Document {
                                data: String::new(),
                                format,
                                media_type,
                                additional_props,
                            }
                        }
                    }
                })
            }
            DocumentBuilderData::Url(url) => {
                let format = self.format;
                let media_type = self.media_type;
                let additional_props = self.additional_props;
                let mut error_handler = self.error_handler;
                
                AsyncTask::new(async move {
                    match reqwest::get(&url).await {
                        Ok(resp) => match resp.text().await {
                            Ok(data) => Document {
                                data,
                                format,
                                media_type,
                                additional_props,
                            },
                            Err(e) => {
                                error_handler(format!("Failed to read response from {}: {}", url, e));
                                Document {
                                    data: String::new(),
                                    format,
                                    media_type,
                                    additional_props,
                                }
                            }
                        },
                        Err(e) => {
                            error_handler(format!("Failed to fetch URL {}: {}", url, e));
                            Document {
                                data: String::new(),
                                format,
                                media_type,
                                additional_props,
                            }
                        }
                    }
                })
            }
            DocumentBuilderData::GitHub(path) => {
                let format = self.format;
                let media_type = self.media_type;
                let additional_props = self.additional_props;
                let mut error_handler = self.error_handler;
                
                // Parse owner/repo/path format
                let parts: Vec<&str> = path.split('/').collect();
                if parts.len() < 3 {
                    error_handler(format!("Invalid GitHub path format: {}", path));
                    return AsyncTask::from_value(Document {
                        data: String::new(),
                        format,
                        media_type,
                        additional_props,
                    });
                }
                
                let owner = parts[0];
                let repo = parts[1];
                let file_path = parts[2..].join("/");
                let api_url = format!(
                    "https://api.github.com/repos/{}/{}/contents/{}",
                    owner, repo, file_path
                );
                
                AsyncTask::new(async move {
                    match reqwest::get(&api_url).await {
                        Ok(resp) => match resp.json::<serde_json::Value>().await {
                            Ok(json) => {
                                if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                                    // GitHub API returns base64 encoded content
                                    match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, content.replace('\n', "")) {
                                        Ok(decoded) => match String::from_utf8(decoded) {
                                            Ok(data) => Document {
                                                data,
                                                format,
                                                media_type,
                                                additional_props,
                                            },
                                            Err(e) => {
                                                error_handler(format!("Invalid UTF-8 in GitHub file: {}", e));
                                                Document {
                                                    data: String::new(),
                                                    format,
                                                    media_type,
                                                    additional_props,
                                                }
                                            }
                                        },
                                        Err(e) => {
                                            error_handler(format!("Failed to decode base64: {}", e));
                                            Document {
                                                data: String::new(),
                                                format,
                                                media_type,
                                                additional_props,
                                            }
                                        }
                                    }
                                } else {
                                    error_handler(format!("No content field in GitHub response"));
                                    Document {
                                        data: String::new(),
                                        format,
                                        media_type,
                                        additional_props,
                                    }
                                }
                            }
                            Err(e) => {
                                error_handler(format!("Failed to parse GitHub response: {}", e));
                                Document {
                                    data: String::new(),
                                    format,
                                    media_type,
                                    additional_props,
                                }
                            }
                        },
                        Err(e) => {
                            error_handler(format!("Failed to fetch from GitHub: {}", e));
                            Document {
                                data: String::new(),
                                format,
                                media_type,
                                additional_props,
                            }
                        }
                    }
                })
            }
            DocumentBuilderData::Glob(pattern) => {
                let format = self.format;
                let media_type = self.media_type;
                let additional_props = self.additional_props;
                let mut error_handler = self.error_handler;
                
                // For glob, we can't return a single Document, this should use stream()
                error_handler(format!("Use stream() for glob patterns, not load_async()"));
                AsyncTask::from_value(Document {
                    data: String::new(),
                    format,
                    media_type,
                    additional_props,
                })
            }
        }
    }
    
    /// Stream documents matching a glob pattern or load a single document as a stream
    pub fn stream(self) -> AsyncStream<Document> {
        match self.data {
            DocumentBuilderData::Glob(pattern) => {
                let format = self.format.clone();
                let media_type = self.media_type.clone();
                let additional_props = self.additional_props.clone();
                let mut error_handler = self.error_handler;
                
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                
                tokio::spawn(async move {
                    match glob::glob(&pattern) {
                        Ok(paths) => {
                            for path_result in paths {
                                match path_result {
                                    Ok(path) => {
                                        match tokio::fs::read_to_string(&path).await {
                                            Ok(data) => {
                                                let doc = Document {
                                                    data,
                                                    format,
                                                    media_type,
                                                    additional_props: additional_props.clone(),
                                                };
                                                if tx.send(doc).is_err() {
                                                    break;
                                                }
                                            }
                                            Err(e) => {
                                                error_handler(format!("Failed to read {:?}: {}", path, e));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error_handler(format!("Glob error: {}", e));
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error_handler(format!("Invalid glob pattern: {}", e));
                        }
                    }
                });
                
                AsyncStream::new(rx)
            }
            _ => {
                // For non-glob sources, create a single-item stream
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                
                tokio::spawn(async move {
                    if let Ok(doc) = self.load_async().await {
                        let _ = tx.send(doc);
                    }
                });
                
                AsyncStream::new(rx)
            }
        }
    }
    
    /// Stream document content in chunks
    pub fn stream_chunks(self, chunk_size: usize) -> AsyncStream<DocumentChunk> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            // First load the document
            let doc = match self.load_async().await {
                Ok(doc) => doc,
                Err(_) => return,
            };
            
            // Then chunk it
            let content = &doc.data;
            let mut offset = 0;
            
            while offset < content.len() {
                let end = (offset + chunk_size).min(content.len());
                let chunk = DocumentChunk::new(&content[offset..end])
                    .with_range(offset, end);
                
                if tx.send(chunk).is_err() {
                    break;
                }
                
                offset = end;
            }
        });
        
        AsyncStream::new(rx)
    }
    
    /// Stream document content line by line
    pub fn stream_lines(self) -> AsyncStream<DocumentChunk> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        
        tokio::spawn(async move {
            // First load the document
            let doc = match self.load_async().await {
                Ok(doc) => doc,
                Err(_) => return,
            };
            
            // Then split by lines
            let mut offset = 0;
            for line in doc.data.lines() {
                let chunk = DocumentChunk::new(line)
                    .with_range(offset, offset + line.len());
                
                if tx.send(chunk).is_err() {
                    break;
                }
                
                offset += line.len() + 1; // +1 for newline
            }
        });
        
        AsyncStream::new(rx)
    }
}
