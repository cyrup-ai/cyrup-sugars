//! Git manager for coordinating release operations.
//!
//! This module provides high-level Git management for release workflows,
//! coordinating commits, tags, pushes, and rollbacks.

use crate::error::{Result, GitError};
use crate::git::{GitOperations, GitRepository, CommitInfo, TagInfo, PushInfo, ValidationResult, ResetType};
use semver::Version;
use std::path::Path;

/// High-level Git manager for release operations
#[derive(Debug)]
pub struct GitManager {
    /// Underlying Git repository
    repository: GitRepository,
    /// Configuration for Git operations
    config: GitConfig,
    /// Release state tracking
    release_state: ReleaseState,
}

/// Configuration for Git operations
#[derive(Debug, Clone)]
pub struct GitConfig {
    /// Default remote name for push operations
    pub default_remote: String,
    /// Whether to create annotated tags
    pub annotated_tags: bool,
    /// Whether to push tags automatically
    pub auto_push_tags: bool,
    /// Custom commit message template
    pub commit_message_template: Option<String>,
    /// Custom tag message template
    pub tag_message_template: Option<String>,
    /// Whether to verify signatures
    pub verify_signatures: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self {
            default_remote: "origin".to_string(),
            annotated_tags: true,
            auto_push_tags: true,
            commit_message_template: None,
            tag_message_template: None,
            verify_signatures: false,
        }
    }
}

/// State tracking for release operations
#[derive(Debug, Clone)]
pub struct ReleaseState {
    /// Commit created for this release
    release_commit: Option<CommitInfo>,
    /// Tag created for this release
    release_tag: Option<TagInfo>,
    /// Whether commits have been pushed
    commits_pushed: bool,
    /// Whether tags have been pushed
    tags_pushed: bool,
    /// Previous HEAD before release (for rollback)
    previous_head: Option<String>,
}

impl Default for ReleaseState {
    fn default() -> Self {
        Self {
            release_commit: None,
            release_tag: None,
            commits_pushed: false,
            tags_pushed: false,
            previous_head: None,
        }
    }
}

/// Result of a complete release operation
#[derive(Debug, Clone)]
pub struct ReleaseResult {
    /// Version that was released
    pub version: Version,
    /// Commit information
    pub commit: CommitInfo,
    /// Tag information
    pub tag: TagInfo,
    /// Push information (if pushed)
    pub push_info: Option<PushInfo>,
    /// Duration of the operation
    pub duration: std::time::Duration,
}

/// Result of a rollback operation
#[derive(Debug, Clone)]
pub struct RollbackResult {
    /// Whether rollback was successful
    pub success: bool,
    /// Operations that were rolled back
    pub rolled_back_operations: Vec<String>,
    /// Any warnings during rollback
    pub warnings: Vec<String>,
    /// Duration of the rollback
    pub duration: std::time::Duration,
}

impl GitManager {
    /// Create a new Git manager for the given repository path
    pub fn new<P: AsRef<Path>>(repo_path: P) -> Result<Self> {
        let repository = GitRepository::open(repo_path)?;
        let config = GitConfig::default();
        let release_state = ReleaseState::default();

        Ok(Self {
            repository,
            config,
            release_state,
        })
    }

    /// Create a Git manager with custom configuration
    pub fn with_config<P: AsRef<Path>>(repo_path: P, config: GitConfig) -> Result<Self> {
        let repository = GitRepository::open(repo_path)?;
        let release_state = ReleaseState::default();

        Ok(Self {
            repository,
            config,
            release_state,
        })
    }

    /// Perform a complete release operation (commit + tag + push)
    pub async fn perform_release(&mut self, version: &Version, push_to_remote: bool) -> Result<ReleaseResult> {
        let start_time = std::time::Instant::now();

        // Note: Validation is performed at the start of the release command,
        // before version files are modified. We don't validate here because
        // the working directory is expected to have modified version files
        // that need to be committed as part of the release.

        // Store current HEAD for potential rollback
        let current_branch = self.repository.get_current_branch().await?;
        self.release_state.previous_head = Some(current_branch.commit_hash);

        // Create release commit
        let commit_message = self.generate_commit_message(version);
        let commit = self.repository.create_release_commit(version, Some(commit_message)).await?;
        self.release_state.release_commit = Some(commit.clone());

        // Create version tag
        let tag_message = self.generate_tag_message(version);
        let tag = self.repository.create_version_tag(version, Some(tag_message)).await?;
        self.release_state.release_tag = Some(tag.clone());

        // Push to remote if requested
        let push_info = if push_to_remote {
            match self.push_release().await {
                Ok(push_info) => {
                    self.release_state.commits_pushed = true;
                    self.release_state.tags_pushed = true;
                    Some(push_info)
                }
                Err(e) => {
                    // If push fails, we might want to rollback
                    return Err(GitError::PushFailed {
                        reason: format!("Failed to push release: {}. Local changes preserved.", e),
                    }.into());
                }
            }
        } else {
            None
        };

        let duration = start_time.elapsed();

        Ok(ReleaseResult {
            version: version.clone(),
            commit,
            tag,
            push_info,
            duration,
        })
    }

    /// Push release commits and tags to remote
    async fn push_release(&self) -> Result<PushInfo> {
        self.repository.push_to_remote(
            Some(&self.config.default_remote),
            self.config.auto_push_tags,
        ).await
    }

    /// Rollback a release operation
    pub async fn rollback_release(&mut self) -> Result<RollbackResult> {
        let start_time = std::time::Instant::now();
        let mut rolled_back_operations = Vec::new();
        let mut warnings = Vec::new();
        let mut success = true;

        // Rollback in reverse order of operations

        // 1. Delete remote tag if it was pushed
        if self.release_state.tags_pushed {
            if let Some(ref tag_info) = self.release_state.release_tag {
                match self.repository.delete_tag(&tag_info.name, true).await {
                    Ok(()) => {
                        rolled_back_operations.push(format!("Deleted remote tag {}", tag_info.name));
                    }
                    Err(e) => {
                        warnings.push(format!("Failed to delete remote tag {}: {}", tag_info.name, e));
                    }
                }
            }
        }

        // 2. Delete local tag
        if let Some(ref tag_info) = self.release_state.release_tag {
            match self.repository.delete_tag(&tag_info.name, false).await {
                Ok(()) => {
                    rolled_back_operations.push(format!("Deleted local tag {}", tag_info.name));
                }
                Err(e) => {
                    warnings.push(format!("Failed to delete local tag {}: {}", tag_info.name, e));
                    success = false;
                }
            }
        }

        // 3. Reset to previous commit if we have it
        if let Some(ref previous_head) = self.release_state.previous_head {
            match self.repository.reset_to_commit(previous_head, ResetType::Hard).await {
                Ok(()) => {
                    rolled_back_operations.push("Reset to previous commit".to_string());
                }
                Err(e) => {
                    warnings.push(format!("Failed to reset to previous commit: {}", e));
                    success = false;
                }
            }
        }

        // Clear release state
        self.release_state = ReleaseState::default();

        let duration = start_time.elapsed();

        Ok(RollbackResult {
            success,
            rolled_back_operations,
            warnings,
            duration,
        })
    }

    /// Generate commit message for release
    fn generate_commit_message(&self, version: &Version) -> String {
        if let Some(ref template) = self.config.commit_message_template {
            template.replace("{version}", &version.to_string())
        } else {
            format!("release: v{}", version)
        }
    }

    /// Generate tag message for release
    fn generate_tag_message(&self, version: &Version) -> String {
        if let Some(ref template) = self.config.tag_message_template {
            template.replace("{version}", &version.to_string())
        } else {
            format!("Release v{}", version)
        }
    }

    /// Check if working directory is clean
    pub async fn is_clean(&self) -> Result<bool> {
        self.repository.is_working_directory_clean().await
    }

    /// Get current branch information
    pub async fn current_branch(&self) -> Result<crate::git::BranchInfo> {
        self.repository.get_current_branch().await
    }

    /// Get recent commit history
    pub async fn recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>> {
        self.repository.get_recent_commits(count).await
    }

    /// Check if a version tag already exists
    pub async fn version_tag_exists(&self, version: &Version) -> Result<bool> {
        let tag_name = format!("v{}", version);
        self.repository.tag_exists(&tag_name).await
    }

    /// Get remote information
    pub async fn remotes(&self) -> Result<Vec<crate::git::RemoteInfo>> {
        self.repository.get_remotes().await
    }

    /// Validate repository for release operations
    pub async fn validate(&self) -> Result<ValidationResult> {
        self.repository.validate_release_readiness().await
    }

    /// Get configuration
    pub fn config(&self) -> &GitConfig {
        &self.config
    }

    /// Update configuration
    pub fn set_config(&mut self, config: GitConfig) {
        self.config = config;
    }

    /// Check if there's an active release
    pub fn has_active_release(&self) -> bool {
        self.release_state.release_commit.is_some() || self.release_state.release_tag.is_some()
    }

    /// Get current release state
    pub fn release_state(&self) -> &ReleaseState {
        &self.release_state
    }

    /// Clear release state (call after successful completion)
    pub fn clear_release_state(&mut self) {
        self.release_state = ReleaseState::default();
    }

    /// Create a backup point before starting operations
    pub async fn create_backup_point(&mut self) -> Result<BackupPoint> {
        let current_branch = self.repository.get_current_branch().await?;
        let recent_commits = self.repository.get_recent_commits(5).await?;
        
        Ok(BackupPoint {
            branch_name: current_branch.name,
            commit_hash: current_branch.commit_hash,
            timestamp: chrono::Utc::now(),
            recent_commits,
        })
    }

    /// Restore from a backup point
    pub async fn restore_from_backup(&self, backup: &BackupPoint) -> Result<()> {
        self.repository.reset_to_commit(&backup.commit_hash, ResetType::Hard).await
    }

    /// Get Git repository statistics
    pub async fn get_repository_stats(&self) -> Result<RepositoryStats> {
        let current_branch = self.repository.get_current_branch().await?;
        let is_clean = self.repository.is_working_directory_clean().await?;
        let remotes = self.repository.get_remotes().await?;
        let recent_commits = self.repository.get_recent_commits(10).await?;

        Ok(RepositoryStats {
            current_branch: current_branch.name,
            is_clean,
            remote_count: remotes.len(),
            recent_commit_count: recent_commits.len(),
            has_upstream: current_branch.upstream.is_some(),
        })
    }
}

/// Backup point for repository state
#[derive(Debug, Clone)]
pub struct BackupPoint {
    /// Branch name at backup time
    pub branch_name: String,
    /// Commit hash at backup time
    pub commit_hash: String,
    /// Timestamp when backup was created
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Recent commits for reference
    pub recent_commits: Vec<CommitInfo>,
}

/// Repository statistics
#[derive(Debug, Clone)]
pub struct RepositoryStats {
    /// Current branch name
    pub current_branch: String,
    /// Whether working directory is clean
    pub is_clean: bool,
    /// Number of configured remotes
    pub remote_count: usize,
    /// Number of recent commits
    pub recent_commit_count: usize,
    /// Whether current branch has upstream
    pub has_upstream: bool,
}

impl ReleaseResult {
    /// Format result for display
    pub fn format_result(&self) -> String {
        let mut result = format!("🎉 Release v{} completed successfully!\n", self.version);
        result.push_str(&format!("📦 Commit: {} ({})\n", self.commit.short_hash, self.commit.message));
        result.push_str(&format!("🏷️  Tag: {}\n", self.tag.name));
        
        if let Some(ref push_info) = self.push_info {
            result.push_str(&format!("📤 Pushed to {}: {} commits, {} tags\n", 
                push_info.remote_name, push_info.commits_pushed, push_info.tags_pushed));
        }
        
        result.push_str(&format!("⏱️  Duration: {:.2}s\n", self.duration.as_secs_f64()));
        
        result
    }
}

impl RollbackResult {
    /// Format rollback result for display
    pub fn format_result(&self) -> String {
        let status = if self.success { "✅" } else { "⚠️" };
        let mut result = format!("{} Rollback completed\n", status);
        
        if !self.rolled_back_operations.is_empty() {
            result.push_str("🔄 Operations rolled back:\n");
            for op in &self.rolled_back_operations {
                result.push_str(&format!("  - {}\n", op));
            }
        }
        
        if !self.warnings.is_empty() {
            result.push_str("⚠️  Warnings:\n");
            for warning in &self.warnings {
                result.push_str(&format!("  - {}\n", warning));
            }
        }
        
        result.push_str(&format!("⏱️  Duration: {:.2}s\n", self.duration.as_secs_f64()));
        
        result
    }
}

impl RepositoryStats {
    /// Format stats for display
    pub fn format_stats(&self) -> String {
        let clean_status = if self.is_clean { "✅ Clean" } else { "❌ Dirty" };
        let upstream_status = if self.has_upstream { "✅ Has upstream" } else { "⚠️ No upstream" };
        
        format!(
            "📊 Repository Stats:\n\
             Branch: {} ({})\n\
             Working Directory: {}\n\
             Remotes: {}\n\
             Recent Commits: {}\n\
             Upstream: {}",
            self.current_branch,
            clean_status,
            clean_status,
            self.remote_count,
            self.recent_commit_count,
            upstream_status
        )
    }
}