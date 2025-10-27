//! sugars_gix - Git operations using gix (Gitoxide)
//!
//! Adapted from kodegen_tools_git - proven production implementation

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use gix::hash::ObjectId;
use thiserror::Error;

// Module declarations
pub mod operations;

// Re-export Git operations we need for release management
pub use operations::{
    commit, CommitOpts, Signature,
};

/// Error types for Git operations
#[derive(Debug, Error)]
pub enum GitError {
    #[error("Gix error: {0}")]
    Gix(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Repository not found at path: {0}")]
    RepoNotFound(PathBuf),

    #[error("Remote `{0}` not found")]
    RemoteNotFound(String),

    #[error("Branch `{0}` not found")]
    BranchNotFound(String),

    #[error("Reference `{0}` not found")]
    ReferenceNotFound(String),

    #[error("Merge conflict: {0}")]
    MergeConflict(String),

    #[error("Unsupported operation: {0}")]
    Unsupported(&'static str),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Channel closed prematurely")]
    ChannelClosed,

    #[error("Operation aborted by user")]
    Aborted,
}

impl From<gix::open::Error> for GitError {
    fn from(e: gix::open::Error) -> Self {
        GitError::Gix(Box::new(e))
    }
}

impl From<gix::discover::Error> for GitError {
    fn from(e: gix::discover::Error) -> Self {
        GitError::Gix(Box::new(e))
    }
}

impl From<gix::init::Error> for GitError {
    fn from(e: gix::init::Error) -> Self {
        GitError::Gix(Box::new(e))
    }
}

impl From<gix::clone::Error> for GitError {
    fn from(e: gix::clone::Error) -> Self {
        GitError::Gix(Box::new(e))
    }
}

/// Convenience result alias.
pub type GitResult<T> = Result<T, GitError>;

/// Strong-typed repository wrapper with cheap cloning.
#[derive(Debug, Clone)]
pub struct RepoHandle {
    inner: gix::Repository,
}

impl RepoHandle {
    /// Create from an existing `gix::Repository`.
    #[inline]
    pub fn new(inner: gix::Repository) -> Self {
        Self { inner }
    }

    /// Access the underlying `gix::Repository` with zero cost.
    #[inline]
    pub fn raw(&self) -> &gix::Repository {
        &self.inner
    }

    /// Clone the underlying repository for use in async tasks.
    #[inline]
    pub fn clone_inner(&self) -> gix::Repository {
        self.inner.clone()
    }
}

/// A unique commit identifier.
pub type CommitId = ObjectId;

/// Lightweight commit metadata.
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub id: CommitId,
    pub author: Signature,
    pub summary: String,
    pub time: DateTime<Utc>,
}
