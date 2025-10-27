//! Git operations and management for release workflows.
//!
//! This module provides comprehensive Git integration using the gix library,
//! offering atomic operations, rollback capabilities, and release coordination.

mod operations;
mod manager;

pub use operations::{
    GitOperations, GitRepository, CommitInfo, TagInfo, PushInfo, BranchInfo, RemoteInfo,
    ResetType, ValidationResult,
};
pub use manager::{
    GitManager, GitConfig, ReleaseResult, RollbackResult, BackupPoint, RepositoryStats,
};

use crate::error::Result;

/// Create a Git manager for the current directory
pub fn create_git_manager() -> Result<GitManager> {
    GitManager::new(".")
}

/// Create a Git manager with custom configuration
pub fn create_git_manager_with_config(config: GitConfig) -> Result<GitManager> {
    GitManager::with_config(".", config)
}

/// Quick validation of Git repository for release
pub async fn quick_git_validation() -> Result<ValidationResult> {
    let repo = GitRepository::open(".")?;
    repo.validate_release_readiness().await
}

/// Check if current directory is a clean Git repository
pub async fn is_git_clean() -> Result<bool> {
    let repo = GitRepository::open(".")?;
    repo.is_working_directory_clean().await
}

/// Get current Git branch information
pub async fn current_git_branch() -> Result<BranchInfo> {
    let repo = GitRepository::open(".")?;
    repo.get_current_branch().await
}

/// Check if a version tag exists
pub async fn version_tag_exists(version: &semver::Version) -> Result<bool> {
    let repo = GitRepository::open(".")?;
    let tag_name = format!("v{}", version);
    repo.tag_exists(&tag_name).await
}

/// Get recent commit history
pub async fn get_recent_commits(count: usize) -> Result<Vec<CommitInfo>> {
    let repo = GitRepository::open(".")?;
    repo.get_recent_commits(count).await
}