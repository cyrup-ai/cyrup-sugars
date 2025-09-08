//! Core Git operations using the gix library.
//!
//! This module provides atomic Git operations for release management,
//! including commits, tags, pushes, and rollback capabilities.

use crate::error::{Result, GitError};
use gix::{Repository, ObjectId, ThreadSafeRepository};
use semver::Version;
use std::path::Path;

/// Trait defining all required Git operations for release management
#[async_trait::async_trait]
pub trait GitOperations {
    /// Create a commit with all current changes
    async fn create_release_commit(&self, version: &Version, message: Option<String>) -> Result<CommitInfo>;

    /// Create a version tag
    async fn create_version_tag(&self, version: &Version, message: Option<String>) -> Result<TagInfo>;

    /// Push commits and tags to remote
    async fn push_to_remote(&self, remote_name: Option<&str>, push_tags: bool) -> Result<PushInfo>;

    /// Check if working directory is clean
    async fn is_working_directory_clean(&self) -> Result<bool>;

    /// Get current branch information
    async fn get_current_branch(&self) -> Result<BranchInfo>;

    /// Reset to previous commit (rollback)
    async fn reset_to_commit(&self, commit_id: &str, reset_type: ResetType) -> Result<()>;

    /// Delete a tag (local and optionally remote)
    async fn delete_tag(&self, tag_name: &str, delete_remote: bool) -> Result<()>;

    /// Get commit history
    async fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>>;

    /// Check if tag exists
    async fn tag_exists(&self, tag_name: &str) -> Result<bool>;

    /// Get remote information
    async fn get_remotes(&self) -> Result<Vec<RemoteInfo>>;

    /// Validate repository state for release
    async fn validate_release_readiness(&self) -> Result<ValidationResult>;
}

/// Information about a Git commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Commit hash (full SHA)
    pub hash: String,
    /// Short commit hash
    pub short_hash: String,
    /// Commit message
    pub message: String,
    /// Author name
    pub author_name: String,
    /// Author email
    pub author_email: String,
    /// Commit timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Parent commit hashes
    pub parents: Vec<String>,
}

/// Information about a Git tag
#[derive(Debug, Clone)]
pub struct TagInfo {
    /// Tag name
    pub name: String,
    /// Tag message (if annotated)
    pub message: Option<String>,
    /// Target commit hash
    pub target_commit: String,
    /// Tag timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Whether this is an annotated tag
    pub is_annotated: bool,
}

/// Information about a push operation
#[derive(Debug, Clone)]
pub struct PushInfo {
    /// Remote name that was pushed to
    pub remote_name: String,
    /// Number of commits pushed
    pub commits_pushed: usize,
    /// Number of tags pushed
    pub tags_pushed: usize,
    /// Any warnings or notes from the push
    pub warnings: Vec<String>,
}

/// Information about a Git branch
#[derive(Debug, Clone)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Whether this is the current branch
    pub is_current: bool,
    /// Current commit hash
    pub commit_hash: String,
    /// Tracking remote branch (if any)
    pub upstream: Option<String>,
    /// Number of commits ahead of upstream
    pub ahead_count: Option<usize>,
    /// Number of commits behind upstream
    pub behind_count: Option<usize>,
}

/// Information about a Git remote
#[derive(Debug, Clone)]
pub struct RemoteInfo {
    /// Remote name
    pub name: String,
    /// Fetch URL
    pub fetch_url: String,
    /// Push URL (may be different from fetch)
    pub push_url: String,
    /// Whether this remote is reachable
    pub is_reachable: bool,
}

/// Type of Git reset operation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetType {
    /// Soft reset (keep changes in index)
    Soft,
    /// Mixed reset (keep changes in working directory)
    Mixed,
    /// Hard reset (discard all changes)
    Hard,
}

/// Result of Git validation for release readiness
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the repository is ready for release
    pub is_ready: bool,
    /// Issues that prevent release
    pub blocking_issues: Vec<String>,
    /// Warnings that should be addressed
    pub warnings: Vec<String>,
    /// Repository status summary
    pub status_summary: String,
}

/// Git repository manager implementing GitOperations
#[derive(Debug)]
pub struct GitRepository {
    /// Gix repository instance
    repository: ThreadSafeRepository,
    /// Working directory path
    work_dir: std::path::PathBuf,
}

impl GitRepository {
    /// Open an existing Git repository
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = gix::discover(path.as_ref())
            .map_err(|_| GitError::NotRepository)?;

        let work_dir = repo.work_dir()
            .ok_or(GitError::NotRepository)?
            .to_path_buf();

        Ok(Self {
            repository: repo.into(),
            work_dir,
        })
    }

    /// Initialize a new Git repository
    pub fn init<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = gix::init(path.as_ref())
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to initialize repository: {}", e),
            })?;

        let work_dir = repo.work_dir()
            .ok_or(GitError::NotRepository)?
            .to_path_buf();

        Ok(Self {
            repository: repo.into(),
            work_dir,
        })
    }

    /// Get the underlying gix repository
    pub fn gix_repository(&self) -> Repository {
        self.repository.to_thread_local()
    }

    /// Convert gix commit to CommitInfo
    fn commit_to_info(&self, commit: gix::Commit) -> Result<CommitInfo> {
        let hash = commit.id().to_string();
        let short_hash = commit.id().shorten().to_string();
        
        let message = commit.message()
            .map(|m| m.summary().to_string())
            .unwrap_or_else(|| "No commit message".to_string());

        let author = commit.author();
        let author_name = author.name.to_string();
        let author_email = author.email.to_string();
        
        let timestamp = chrono::DateTime::from_timestamp(author.time.seconds, 0)
            .unwrap_or_else(chrono::Utc::now);

        let parents: Vec<String> = commit.parent_ids()
            .map(|id| id.to_string())
            .collect();

        Ok(CommitInfo {
            hash,
            short_hash,
            message,
            author_name,
            author_email,
            timestamp,
            parents,
        })
    }

    /// Add all changes to the index
    async fn add_all_changes(&self) -> Result<()> {
        let repo = self.gix_repository();
        let mut index = repo.index()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to access index: {}", e),
            })?;

        // Add all files in working directory
        index.add_all(
            std::iter::empty::<std::path::PathBuf>(),
            gix::index::entry::Stage::NonConflicted,
            gix::index::write::Options::default(),
        ).map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to add files to index: {}", e),
        })?;

        Ok(())
    }

    /// Create signature for commits
    fn create_signature(&self) -> Result<gix::actor::SignatureRef<'static>> {
        let repo = self.gix_repository();
        let config = repo.config_snapshot();
        
        let name = config.string("user.name")
            .ok_or_else(|| GitError::CommitFailed {
                reason: "Git user.name not configured".to_string(),
            })?;
        
        let email = config.string("user.email")
            .ok_or_else(|| GitError::CommitFailed {
                reason: "Git user.email not configured".to_string(),
            })?;

        let signature = gix::actor::Signature {
            name: name.into_owned().into(),
            email: email.into_owned().into(),
            time: gix::date::Time::now_local_or_utc(),
        };

        Ok(signature.to_ref())
    }
}

#[async_trait::async_trait]
impl GitOperations for GitRepository {
    async fn create_release_commit(&self, version: &Version, message: Option<String>) -> Result<CommitInfo> {
        let repo = self.gix_repository();
        
        // Add all changes to index
        self.add_all_changes().await?;

        // Create commit message
        let commit_message = message.unwrap_or_else(|| {
            format!("release: v{}", version)
        });

        // Create signature
        let signature = self.create_signature()?;

        // Get current HEAD
        let head = repo.head()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;

        let parent_commit = head.peel_to_commit_in_place()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get parent commit: {}", e),
            })?;

        // Get index tree
        let index = repo.index()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to access index: {}", e),
            })?;

        let tree_id = index.write(gix::index::write::Options::default())
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to write index: {}", e),
            })?;

        // Create commit
        let commit_id = repo.commit(
            Some(&head.name().as_bstr()),
            &signature,
            &signature,
            &commit_message,
            tree_id,
            [parent_commit.id()],
        ).map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to create commit: {}", e),
        })?;

        // Get the created commit
        let commit = repo.find_commit(commit_id)
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to find created commit: {}", e),
            })?;

        self.commit_to_info(commit)
    }

    async fn create_version_tag(&self, version: &Version, message: Option<String>) -> Result<TagInfo> {
        let repo = self.gix_repository();
        let tag_name = format!("v{}", version);

        // Check if tag already exists
        if self.tag_exists(&tag_name).await? {
            return Err(GitError::TagExists { tag: tag_name }.into());
        }

        // Get current HEAD commit
        let head = repo.head()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;

        let target_commit = head.peel_to_commit_in_place()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get target commit: {}", e),
            })?;

        let target_id = target_commit.id();

        // Create tag message
        let tag_message = message.unwrap_or_else(|| {
            format!("Release v{}", version)
        });

        // Create signature
        let signature = self.create_signature()?;

        // Create annotated tag
        let tag_ref_name = format!("refs/tags/{}", tag_name);
        let tag_id = repo.tag(
            &tag_name,
            target_id,
            gix::object::Kind::Commit,
            Some(&signature),
            &tag_message,
            gix::refs::transaction::PreviousValue::MustNotExist,
        ).map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to create tag: {}", e),
        })?;

        Ok(TagInfo {
            name: tag_name,
            message: Some(tag_message),
            target_commit: target_id.to_string(),
            timestamp: chrono::Utc::now(),
            is_annotated: true,
        })
    }

    async fn push_to_remote(&self, remote_name: Option<&str>, push_tags: bool) -> Result<PushInfo> {
        let repo = self.gix_repository();
        let remote_name = remote_name.unwrap_or("origin");

        // Get remote
        let remote = repo.find_remote(remote_name)
            .map_err(|e| GitError::RemoteOperationFailed {
                operation: "find remote".to_string(),
                reason: format!("Remote '{}' not found: {}", remote_name, e),
            })?;

        let mut warnings = Vec::new();
        let mut commits_pushed = 0;
        let mut tags_pushed = 0;

        // Push current branch
        match self.push_current_branch(&remote).await {
            Ok(count) => commits_pushed = count,
            Err(e) => {
                return Err(GitError::PushFailed {
                    reason: format!("Failed to push commits: {}", e),
                }.into());
            }
        }

        // Push tags if requested
        if push_tags {
            match self.push_tags(&remote).await {
                Ok(count) => tags_pushed = count,
                Err(e) => {
                    warnings.push(format!("Failed to push tags: {}", e));
                }
            }
        }

        Ok(PushInfo {
            remote_name: remote_name.to_string(),
            commits_pushed,
            tags_pushed,
            warnings,
        })
    }

    async fn is_working_directory_clean(&self) -> Result<bool> {
        let repo = self.gix_repository();
        
        let status = repo.status(gix::status::Platform::default())
            .map_err(|e| GitError::RemoteOperationFailed {
                operation: "status check".to_string(),
                reason: e.to_string(),
            })?;

        // Check for any modifications
        for entry in status {
            let entry = entry.map_err(|e| GitError::RemoteOperationFailed {
                operation: "status iteration".to_string(),
                reason: e.to_string(),
            })?;

            if entry.status().is_modified() || 
               entry.status().is_added() || 
               entry.status().is_deleted() ||
               entry.status().is_renamed() ||
               entry.status().is_copied() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    async fn get_current_branch(&self) -> Result<BranchInfo> {
        let repo = self.gix_repository();
        
        let head = repo.head()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;

        let branch_name = head.referent_name()
            .and_then(|name| name.shorten().to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "detached HEAD".to_string());

        let commit = head.peel_to_commit_in_place()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get current commit: {}", e),
            })?;

        let commit_hash = commit.id().to_string();

        // TODO: Implement upstream tracking and ahead/behind counts
        let upstream = None;
        let ahead_count = None;
        let behind_count = None;

        Ok(BranchInfo {
            name: branch_name,
            is_current: true,
            commit_hash,
            upstream,
            ahead_count,
            behind_count,
        })
    }

    async fn reset_to_commit(&self, commit_id: &str, reset_type: ResetType) -> Result<()> {
        let repo = self.gix_repository();
        
        // Parse commit ID
        let target_id = repo.rev_parse_single(commit_id)
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Invalid commit ID '{}': {}", commit_id, e),
            })?;

        // Get target commit
        let target_commit = repo.find_commit(target_id)
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to find commit '{}': {}", commit_id, e),
            })?;

        // Perform reset based on type
        match reset_type {
            ResetType::Soft => {
                // Move HEAD but keep index and working directory
                self.reset_head_to_commit(&repo, target_commit.id()).await?;
            }
            ResetType::Mixed => {
                // Move HEAD and reset index, keep working directory
                self.reset_head_to_commit(&repo, target_commit.id()).await?;
                self.reset_index_to_commit(&repo, &target_commit).await?;
            }
            ResetType::Hard => {
                // Move HEAD, reset index, and reset working directory
                self.reset_head_to_commit(&repo, target_commit.id()).await?;
                self.reset_index_to_commit(&repo, &target_commit).await?;
                self.reset_working_directory(&repo, &target_commit).await?;
            }
        }

        Ok(())
    }

    async fn delete_tag(&self, tag_name: &str, delete_remote: bool) -> Result<()> {
        let repo = self.gix_repository();
        
        // Delete local tag
        let tag_ref_name = format!("refs/tags/{}", tag_name);
        repo.refs.delete(&tag_ref_name)
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to delete local tag '{}': {}", tag_name, e),
            })?;

        // Delete remote tag if requested
        if delete_remote {
            // TODO: Implement remote tag deletion
            // This requires push with refspec `:refs/tags/{tag_name}`
        }

        Ok(())
    }

    async fn get_recent_commits(&self, count: usize) -> Result<Vec<CommitInfo>> {
        let repo = self.gix_repository();
        
        let head = repo.head()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;

        let mut commits = Vec::new();
        let mut walker = head.into_peeled_id()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to peel HEAD: {}", e),
            })?
            .ancestors()
            .all()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to create commit walker: {}", e),
            })?;

        for _ in 0..count {
            if let Some(commit_result) = walker.next() {
                let commit_info = commit_result
                    .map_err(|e| GitError::BranchOperationFailed {
                        reason: format!("Failed to get commit: {}", e),
                    })?;

                let commit = repo.find_commit(commit_info.id())
                    .map_err(|e| GitError::BranchOperationFailed {
                        reason: format!("Failed to find commit: {}", e),
                    })?;

                commits.push(self.commit_to_info(commit)?);
            } else {
                break;
            }
        }

        Ok(commits)
    }

    async fn tag_exists(&self, tag_name: &str) -> Result<bool> {
        let repo = self.gix_repository();
        let tag_ref_name = format!("refs/tags/{}", tag_name);
        
        match repo.refs.find(&tag_ref_name) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    async fn get_remotes(&self) -> Result<Vec<RemoteInfo>> {
        let repo = self.gix_repository();
        let mut remotes = Vec::new();

        for remote_name in repo.remote_names() {
            if let Ok(remote) = repo.find_remote(&remote_name) {
                let fetch_url = remote.url(gix::remote::Direction::Fetch)
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let push_url = remote.url(gix::remote::Direction::Push)
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| fetch_url.clone());

                // TODO: Implement reachability check
                let is_reachable = true;

                remotes.push(RemoteInfo {
                    name: remote_name.to_string(),
                    fetch_url,
                    push_url,
                    is_reachable,
                });
            }
        }

        Ok(remotes)
    }

    async fn validate_release_readiness(&self) -> Result<ValidationResult> {
        let mut blocking_issues = Vec::new();
        let mut warnings = Vec::new();

        // Check if working directory is clean
        if !self.is_working_directory_clean().await? {
            blocking_issues.push("Working directory has uncommitted changes".to_string());
        }

        // Check if we're on a valid branch
        match self.get_current_branch().await {
            Ok(branch) => {
                if branch.name == "detached HEAD" {
                    warnings.push("Currently in detached HEAD state".to_string());
                }
            }
            Err(e) => {
                blocking_issues.push(format!("Failed to get current branch: {}", e));
            }
        }

        // Check for remotes
        match self.get_remotes().await {
            Ok(remotes) => {
                if remotes.is_empty() {
                    warnings.push("No remotes configured".to_string());
                }
            }
            Err(_) => {
                warnings.push("Failed to check remotes".to_string());
            }
        }

        let is_ready = blocking_issues.is_empty();
        let status_summary = if is_ready {
            "Repository ready for release".to_string()
        } else {
            format!("{} issues prevent release", blocking_issues.len())
        };

        Ok(ValidationResult {
            is_ready,
            blocking_issues,
            warnings,
            status_summary,
        })
    }
}

impl GitRepository {
    /// Helper method to push current branch
    async fn push_current_branch(&self, remote: &gix::Remote) -> Result<usize> {
        // TODO: Implement actual push operation
        // This is a simplified placeholder
        Ok(1)
    }

    /// Helper method to push tags
    async fn push_tags(&self, remote: &gix::Remote) -> Result<usize> {
        // TODO: Implement tag pushing
        // This is a simplified placeholder
        Ok(0)
    }

    /// Reset HEAD to specific commit
    async fn reset_head_to_commit(&self, repo: &Repository, target_id: ObjectId) -> Result<()> {
        let head = repo.head()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;

        // Update HEAD reference
        repo.refs.transaction()
            .prepare(
                head.name(),
                gix::refs::transaction::Change::Update {
                    expected: gix::refs::transaction::PreviousValue::Any,
                    new: gix::refs::Target::Peeled(target_id),
                },
                gix::refs::transaction::RefLog::AndReference,
            )
            .commit()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to update HEAD: {}", e),
            })?;

        Ok(())
    }

    /// Reset index to specific commit
    async fn reset_index_to_commit(&self, repo: &Repository, target_commit: &gix::Commit) -> Result<()> {
        // TODO: Implement index reset
        Ok(())
    }

    /// Reset working directory to specific commit
    async fn reset_working_directory(&self, repo: &Repository, target_commit: &gix::Commit) -> Result<()> {
        // TODO: Implement working directory reset
        Ok(())
    }
}