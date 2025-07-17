//! Context trait and implementations for loading documents - EXACT API from ARCHITECTURE.md

use crate::domain::Document;
use crate::ZeroOneOrMany;
use std::marker::PhantomData;
use std::path::Path;

/// Marker types for Context
pub struct File;
pub struct Files;
pub struct Directory;
pub struct Github;

/// Context trait - base trait for all context types
pub trait ContextSource: Send + Sync + 'static {
    fn into_documents(self) -> ZeroOneOrMany<Document>;
}

/// Context wrapper with type parameter
pub struct Context<T> {
    _phantom: PhantomData<T>,
    source: Box<dyn ContextSource>,
}

impl<T> Context<T> {
    fn new(source: impl ContextSource) -> Self {
        Self {
            _phantom: PhantomData,
            source: Box::new(source),
        }
    }
}

// Context<File> implementation
impl Context<File> {
    /// Load a single file - EXACT syntax: Context<File>::of("/path/to/file.pdf")
    pub fn of(path: impl AsRef<Path>) -> Self {
        Self::new(FileContext {
            path: path.as_ref().to_path_buf(),
        })
    }
}

struct FileContext {
    path: std::path::PathBuf,
}

impl ContextSource for FileContext {
    fn into_documents(self) -> ZeroOneOrMany<Document> {
        ZeroOneOrMany::One(Document::from_file(self.path).load())
    }
}

// Context<Files> implementation
impl Context<Files> {
    /// Glob pattern for files - EXACT syntax: Context<Files>::glob("/path/**/*.{md,txt}")
    pub fn glob(pattern: impl AsRef<str>) -> Self {
        Self::new(FilesContext {
            pattern: pattern.as_ref().to_string(),
        })
    }
}

struct FilesContext {
    pattern: String,
}

impl ContextSource for FilesContext {
    fn into_documents(self) -> ZeroOneOrMany<Document> {
        // Use glob crate to find files
        glob::glob(&self.pattern)
            .ok()
            .map(|paths| {
                let docs: Vec<Document> = paths
                    .filter_map(Result::ok)
                    .map(|path| Document::from_file(path).load())
                    .collect();
                ZeroOneOrMany::from_vec(docs)
            })
            .unwrap_or(ZeroOneOrMany::None)
    }
}

// Context<Directory> implementation
impl Context<Directory> {
    /// Load all files from directory - EXACT syntax: Context<Directory>::of("/path/to/dir")
    pub fn of(path: impl AsRef<Path>) -> Self {
        Self::new(DirectoryContext {
            path: path.as_ref().to_path_buf(),
        })
    }
}

struct DirectoryContext {
    path: std::path::PathBuf,
}

impl ContextSource for DirectoryContext {
    fn into_documents(self) -> ZeroOneOrMany<Document> {
        // Read all files in directory
        std::fs::read_dir(&self.path)
            .ok()
            .map(|entries| {
                let docs: Vec<Document> = entries
                    .filter_map(Result::ok)
                    .filter_map(|entry| {
                        let path = entry.path();
                        if path.is_file() {
                            Some(Document::from_file(path).load())
                        } else {
                            None
                        }
                    })
                    .collect();
                ZeroOneOrMany::from_vec(docs)
            })
            .unwrap_or(ZeroOneOrMany::None)
    }
}

// Context<Github> implementation
impl Context<Github> {
    /// Glob pattern for GitHub files - EXACT syntax: Context<Github>::glob("/repo/**/*.{rs,md}")
    pub fn glob(pattern: impl AsRef<str>) -> Self {
        Self::new(GithubContext {
            pattern: pattern.as_ref().to_string(),
        })
    }
}

struct GithubContext {
    pattern: String,
}

impl ContextSource for GithubContext {
    fn into_documents(self) -> ZeroOneOrMany<Document> {
        // TODO: Actual GitHub implementation would use GitHub API
        // For now, return None
        ZeroOneOrMany::None
    }
}

// Implement Send + Sync for Context
unsafe impl<T> Send for Context<T> {}
unsafe impl<T> Sync for Context<T> {}