//! Core Git operations using the gix library.
//!
//! This module provides atomic Git operations for release management,
//! including commits, tags, pushes, and rollback capabilities.

use crate::error::{Result, GitError};
use gix::bstr::ByteSlice;
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

        let work_dir = repo.workdir()
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

        let work_dir = repo.workdir()
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
    fn commit_to_info(&self, commit: gix::Commit<'_>) -> Result<CommitInfo> {
        let hash = commit.id().to_string();
        let short_hash = commit.id().shorten()
            .map(|prefix| prefix.to_string())
            .unwrap_or_else(|_| hash.clone());
        
        let message = commit.message()
            .map(|m| m.summary().to_string())
            .unwrap_or_else(|_| "No commit message".to_string());

        let author = commit.author().map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to get author: {}", e),
        })?;
        let author_name = author.name.to_string();
        let author_email = author.email.to_string();
        
        // Get commit time (separate from author)
        let time = commit.time().map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to get commit time: {}", e),
        })?;
        
        // Convert gix time to chrono DateTime
        let timestamp = {
            use chrono::TimeZone;
            chrono::Utc.timestamp_opt(time.seconds, 0)
                .single()
                .ok_or_else(|| GitError::CommitFailed {
                    reason: format!("Invalid timestamp {} for commit", time.seconds),
                })?
        };

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

    /// Create signature for commits
    fn create_signature(&self) -> Result<gix::actor::Signature> {
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

        Ok(signature)
    }
}

#[async_trait::async_trait]
impl GitOperations for GitRepository {
    async fn create_release_commit(&self, version: &Version, message: Option<String>) -> Result<CommitInfo> {
        // Use proven kodegen implementation (now sugars_gix)
        let repo_handle = sugars_gix::RepoHandle::new(self.gix_repository().clone());
        let commit_message = message.unwrap_or_else(|| format!("release: v{}", version));
        
        let opts = sugars_gix::CommitOpts::message(commit_message)
            .all(true);  // Stage all changes
        
        let commit_id = sugars_gix::commit(repo_handle.clone(), opts).await
            .map_err(|e| GitError::CommitFailed {
                reason: format!("kodegen commit failed: {}", e),
            })?;
        
        // Convert to our CommitInfo type
        let repo = self.gix_repository();
        let commit = repo.find_commit(commit_id)
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to find created commit: {}", e),
            })?;
        
        let author = commit.author().map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to get author: {}", e),
        })?;
        let message_obj = commit.message().map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to get message: {}", e),
        })?;
        let summary = message_obj.summary().to_string();
        
        let parent_ids: Vec<String> = commit.parent_ids()
            .map(|id| id.to_string())
            .collect();
        
        Ok(CommitInfo {
            hash: commit_id.to_string(),
            short_hash: commit_id.to_string()[..7].to_string(),
            message: summary,
            author_name: author.name.to_string(),
            author_email: author.email.to_string(),
            timestamp: chrono::Utc::now(),
            parents: parent_ids,
        })
    }
    
    // OLD BROKEN IMPLEMENTATION - REMOVED

    async fn create_version_tag(&self, version: &Version, message: Option<String>) -> Result<TagInfo> {
        let repo = self.gix_repository();
        let tag_name = format!("v{}", version);
        
        // Get HEAD commit ID as target
        let target = repo.head_id()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get HEAD commit: {}", e),
            })?
            .detach();
        
        // Create signature
        let signature = self.create_signature()?;
        
        // Build tag message
        let tag_message = message.unwrap_or_else(|| format!("Release v{}", version));
        
        // Convert signature time to string for SignatureRef
        use gix::bstr::ByteSlice;
        let time_str = signature.time.to_string();
        let sig_ref = gix::actor::SignatureRef {
            name: signature.name.as_bstr(),
            email: signature.email.as_bstr(),
            time: &time_str,
        };
        
        // Create annotated tag
        repo.tag(
            &tag_name,
            target,
            gix::objs::Kind::Commit,
            Some(sig_ref),
            &tag_message,
            gix::refs::transaction::PreviousValue::MustNotExist,
        )
        .map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to create tag {}: {}", tag_name, e),
        })?;
        
        // Get commit for timestamp
        let commit = repo.find_commit(target)
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to find target commit for tag {}: {}", tag_name, e),
            })?;
        
        let commit_time = commit.time()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get commit time for tag {}: {}", tag_name, e),
            })?;
        
        use chrono::TimeZone;
        let timestamp = chrono::Utc.timestamp_opt(commit_time.seconds, 0)
            .single()
            .unwrap_or_else(chrono::Utc::now);
        
        Ok(TagInfo {
            name: tag_name,
            message: Some(tag_message),
            target_commit: target.to_string(),
            timestamp,
            is_annotated: true,
        })
    }

    async fn push_to_remote(&self, remote_name: Option<&str>, push_tags: bool) -> Result<PushInfo> {
        // Note: This uses the git command-line tool rather than gix library calls
        // because gix does not yet support push operations
        
        let remote = remote_name.unwrap_or("origin");
        let mut warnings = Vec::new();
        
        // Get workdir
        let work_dir = self.work_dir.clone();
        
        // Build git push command
        let mut cmd = tokio::process::Command::new("git");
        cmd.current_dir(&work_dir);
        cmd.arg("push");
        
        // Prevent credential prompts from hanging
        cmd.env("GIT_TERMINAL_PROMPT", "0");
        
        // Force English output for consistent parsing
        cmd.env("LC_ALL", "C");
        cmd.env("LANG", "C");
        
        // Capture stdout and stderr
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        
        // Add --tags if requested
        if push_tags {
            cmd.arg("--tags");
        }
        
        // Add remote name
        cmd.arg(remote);
        
        // Spawn and wait with timeout
        let timeout_duration = tokio::time::Duration::from_secs(300);
        let mut child = cmd.spawn()
            .map_err(|e| GitError::RemoteOperationFailed {
                operation: "push".to_string(),
                reason: format!("Failed to spawn git command: {}", e),
            })?;
        
        let status = tokio::select! {
            result = child.wait() => {
                result.map_err(|e| GitError::RemoteOperationFailed {
                    operation: "push".to_string(),
                    reason: format!("Failed to wait for git command: {}", e),
                })?
            }
            () = tokio::time::sleep(timeout_duration) => {
                let _ = child.kill().await;
                return Err(crate::error::ReleaseError::Git(GitError::RemoteOperationFailed {
                    operation: "push".to_string(),
                    reason: "Push operation timed out after 300 seconds".to_string(),
                }));
            }
        };
        
        // Read stderr for any messages
        if let Some(mut stderr) = child.stderr.take() {
            use tokio::io::AsyncReadExt;
            let mut stderr_data = Vec::new();
            let _ = stderr.read_to_end(&mut stderr_data).await;
            if !stderr_data.is_empty() {
                if let Ok(stderr_str) = String::from_utf8(stderr_data) {
                    if !stderr_str.trim().is_empty() {
                        warnings.push(stderr_str);
                    }
                }
            }
        }
        
        if !status.success() {
            return Err(crate::error::ReleaseError::Git(GitError::RemoteOperationFailed {
                operation: "push".to_string(),
                reason: format!("Push failed with exit code {:?}", status.code()),
            }));
        }

        // Parse git output to count actual refs pushed
        let mut stdout_data = Vec::new();
        if let Some(mut stdout) = child.stdout.take() {
            use tokio::io::AsyncReadExt;
            let _ = stdout.read_to_end(&mut stdout_data).await;
        }

        // Combine stdout and stderr for parsing (git sends push info to stderr)
        let combined = format!(
            "{}\n{}",
            String::from_utf8_lossy(&stdout_data),
            warnings.join("\n")
        );

        // Count successful ref updates by parsing output lines
        // Lines with " -> " indicate ref updates (branches/tags)
        let refs_pushed = combined
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                if !trimmed.contains(" -> ") {
                    return false;
                }
                // Exclude errors and rejections
                if trimmed.starts_with('!')
                    || trimmed.starts_with("error:")
                    || trimmed.contains("[rejected]")
                {
                    return false;
                }
                // Match successful update patterns:
                // - "abc123..def456 main -> main" (fast-forward)
                // - " * [new branch] feature -> feature" (new branch)
                // - " * [new tag] v1.0.0 -> v1.0.0" (new tag)
                // - "+ abc123...def456 main -> main (forced)" (force push)
                trimmed.starts_with(|c: char| c.is_ascii_hexdigit())
                    || trimmed.starts_with("* [new")
                    || trimmed.starts_with('+')
            })
            .count();

        // Estimate split between commits and tags
        // If push_tags was specified, count lines with "tag" in them
        let (commits_pushed, tags_pushed) = if push_tags && refs_pushed > 0 {
            let tag_count = combined
                .lines()
                .filter(|line| {
                    let trimmed = line.trim_start();
                    trimmed.contains(" -> ")
                        && trimmed.contains("tag")
                        && !trimmed.starts_with('!')
                        && !trimmed.contains("[rejected]")
                })
                .count();
            (refs_pushed.saturating_sub(tag_count), tag_count)
        } else {
            (refs_pushed, 0)
        };

        Ok(PushInfo {
            remote_name: remote.to_string(),
            commits_pushed,
            tags_pushed,
            warnings,
        })
    }

    async fn is_working_directory_clean(&self) -> Result<bool> {
        let repo = self.gix_repository();
        
        // Use is_dirty() which is the proper API for checking if repo has changes
        let is_dirty = repo.is_dirty()
            .map_err(|e| GitError::RemoteOperationFailed {
                operation: "status check".to_string(),
                reason: e.to_string(),
            })?;
        
        Ok(!is_dirty)
    }

    async fn get_current_branch(&self) -> Result<BranchInfo> {
        let repo = self.gix_repository();
        
        let mut head = repo.head()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;

        let branch_name = head.referent_name()
            .and_then(|name| name.shorten().to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "detached HEAD".to_string());

        let commit = head.peel_to_commit_in_place()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get current commit: {}", e),
            })?;

        let commit_hash = commit.id().to_string();

        // Get upstream tracking information and ahead/behind counts
        let (upstream, ahead_count, behind_count) = self.get_upstream_info(&repo, &mut head)?;

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
        
        // Resolve target commit ID
        use gix::bstr::ByteSlice;
        let target_id = repo.rev_parse_single(commit_id.as_bytes().as_bstr())
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Invalid commit ID '{}': {}", commit_id, e),
            })?;
        
        // Get target commit
        let target_commit = repo.find_object(target_id)
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to find commit: {}", e),
            })?
            .try_into_commit()
            .map_err(|_| GitError::BranchOperationFailed {
                reason: "Target is not a commit".to_string(),
            })?;
        
        let target_obj_id = target_id.detach();
        
        // Execute reset in safe order: working directory -> index -> HEAD
        // This minimizes risk of inconsistent state
        
        // Step 1: Reset working directory (if Hard reset)
        if reset_type == ResetType::Hard {
            self.reset_working_directory(&repo, &target_commit)?;
        }
        
        // Step 2: Reset index (if Mixed or Hard reset)
        if reset_type == ResetType::Mixed || reset_type == ResetType::Hard {
            self.reset_index_to_commit(&repo, &target_commit)?;
        }
        
        // Step 3: Move HEAD (all reset types)
        self.reset_head_to_commit(&repo, target_obj_id, commit_id)?;
        
        Ok(())
    }

    async fn delete_tag(&self, tag_name: &str, delete_remote: bool) -> Result<()> {
        let repo = self.gix_repository();
        let tag_ref_name = format!("refs/tags/{}", tag_name);
        
        // Check if tag exists
        use gix::bstr::ByteSlice;
        repo.refs.find(tag_ref_name.as_bytes().as_bstr())
            .map_err(|_| GitError::BranchOperationFailed {
                reason: format!("Tag '{}' not found", tag_name),
            })?;
        
        // Delete the tag using transaction API
        let ref_name = gix::refs::FullName::try_from(tag_ref_name.as_bytes().as_bstr())
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Invalid ref name for tag '{}': {}", tag_name, e),
            })?;
        
        let edit = gix::refs::transaction::RefEdit {
            change: gix::refs::transaction::Change::Delete {
                expected: gix::refs::transaction::PreviousValue::Any,
                log: gix::refs::transaction::RefLog::AndReference,
            },
            name: ref_name,
            deref: false,
        };
        
        repo.refs.transaction()
            .prepare(
                vec![edit],
                gix::lock::acquire::Fail::Immediately,
                gix::lock::acquire::Fail::Immediately,
            )
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to prepare tag deletion transaction for '{}': {}", tag_name, e),
            })?
            .commit(None)
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to commit tag deletion transaction for '{}': {}", tag_name, e),
            })?;
        
        // Handle remote tag deletion if requested
        if delete_remote {
            // Remote deletion requires git CLI since gix doesn't support push operations
            self.delete_remote_tag_cli(tag_name).await?;
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
            if let Ok(remote) = repo.find_remote(&*remote_name) {
                let fetch_url = remote.url(gix::remote::Direction::Fetch)
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let push_url = remote.url(gix::remote::Direction::Push)
                    .map(|url| url.to_string())
                    .unwrap_or_else(|| fetch_url.clone());

                remotes.push(RemoteInfo {
                    name: remote_name.to_string(),
                    fetch_url,
                    push_url,
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
    /// Reset HEAD to specific commit
    fn reset_head_to_commit(&self, repo: &Repository, target_id: ObjectId, target_ref: &str) -> Result<()> {
        let head = repo.head()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to get HEAD: {}", e),
            })?;
        
        // Check if HEAD is symbolic (on a branch) or detached
        let is_symbolic = matches!(
            head.kind,
            gix::head::Kind::Symbolic(_) | gix::head::Kind::Unborn(_)
        );
        
        if is_symbolic {
            // Symbolic HEAD: Update the branch reference that HEAD points to
            use gix::bstr::ByteSlice;
            let head_name = head.name().as_bstr();
            let ref_name = gix::refs::FullName::try_from(head_name.as_bstr())
                .map_err(|e| GitError::BranchOperationFailed {
                    reason: format!("Invalid ref name: {}", e),
                })?;
            
            use gix::refs::Target;
            use gix::refs::transaction::{Change, LogChange, PreviousValue, RefEdit, RefLog};
            
            repo.edit_reference(RefEdit {
                change: Change::Update {
                    log: LogChange {
                        mode: RefLog::AndReference,
                        force_create_reflog: false,
                        message: format!("reset: moving to {}", target_ref).into(),
                    },
                    expected: PreviousValue::Any,
                    new: Target::Object(target_id),
                },
                name: ref_name,
                deref: true,
            })
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to update reference: {}", e),
            })?;
        } else {
            // Detached HEAD: Update HEAD directly
            use gix::refs::transaction::PreviousValue;
            
            repo.reference(
                "HEAD",
                target_id,
                PreviousValue::Any,
                format!("reset: moving to {}", target_ref),
            )
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to update HEAD: {}", e),
            })?;
        }
        
        Ok(())
    }

    /// Reset index to specific commit
    fn reset_index_to_commit(&self, repo: &Repository, target_commit: &gix::Commit<'_>) -> Result<()> {
        // Get tree ID from target commit
        let tree_id = target_commit.tree_id()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get tree ID: {}", e),
            })?;
        
        // Create new index from target tree
        let mut new_index = repo.index_from_tree(&tree_id)
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to create index from tree: {}", e),
            })?;
        
        // Write new index to disk with proper locking and checksum
        use gix::index::write::Options;
        new_index.write(Options::default())
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to write index: {}", e),
            })?;
        
        Ok(())
    }

    /// Reset working directory to specific commit
    fn reset_working_directory(&self, repo: &Repository, target_commit: &gix::Commit<'_>) -> Result<()> {
        // Get tree ID from target commit
        let tree_id = target_commit.tree_id()
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to get tree ID: {}", e),
            })?;
        
        // Create index from target tree
        let mut index = repo.index_from_tree(&tree_id)
            .map_err(|e| GitError::CommitFailed {
                reason: format!("Failed to create index from tree: {}", e),
            })?;
        
        // Get worktree path
        let worktree = repo.worktree()
            .ok_or_else(|| GitError::CommitFailed {
                reason: "Cannot reset working directory in bare repository".to_string(),
            })?;
        let worktree_path = worktree.base().to_owned();
        
        // Configure checkout options for force overwrite
        let mut checkout_opts = repo.checkout_options(
            gix::worktree::stack::state::attributes::Source::WorktreeThenIdMapping
        )
        .map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to create checkout options: {}", e),
        })?;
        
        // Force overwrite all files (hard reset behavior)
        checkout_opts.overwrite_existing = true;
        checkout_opts.destination_is_initially_empty = false;
        
        // Perform the checkout
        let cancel_token = std::sync::atomic::AtomicBool::new(false);
        let outcome = gix::worktree::state::checkout(
            &mut index,
            &worktree_path,
            repo.objects.clone().into_arc()
                .map_err(|e| GitError::CommitFailed {
                    reason: format!("Failed to get object store: {}", e),
                })?,
            &gix::progress::Discard,
            &gix::progress::Discard,
            &cancel_token,
            checkout_opts,
        )
        .map_err(|e| GitError::CommitFailed {
            reason: format!("Failed to checkout: {}", e),
        })?;
        
        // Check for errors
        if !outcome.errors.is_empty() {
            let error_details: Vec<String> = outcome.errors
                .iter()
                .take(5)
                .map(|err| {
                    let path_str = std::str::from_utf8(err.path.as_ref())
                        .unwrap_or("<invalid utf8>");
                    format!("{}: {}", path_str, err.error)
                })
                .collect();
            
            return Err(crate::error::ReleaseError::Git(GitError::CommitFailed {
                reason: format!("Reset failed with {} error(s): {}", 
                    outcome.errors.len(), 
                    error_details.join(", ")
                ),
            }));
        }
        
        Ok(())
    }
}

// Private helper methods for GitRepository
impl GitRepository {
    /// Get upstream tracking information for current branch
    fn get_upstream_info(
        &self,
        repo: &gix::Repository,
        head: &mut gix::Head<'_>,
    ) -> Result<(Option<String>, Option<usize>, Option<usize>)> {
        // Try to get upstream branch from config
        let upstream = if let Some(branch_ref) = head.referent_name() {
            let branch_name = branch_ref.shorten();
            let config = repo.config_snapshot();
            let branch_section = format!("branch.{branch_name}");

            let remote_name = config
                .string(format!("{branch_section}.remote"))
                .map(|s| s.to_string());

            let merge_ref = config
                .string(format!("{branch_section}.merge"))
                .map(|s| s.to_string());

            if let (Some(remote), Some(merge)) = (remote_name, merge_ref) {
                Some(format!(
                    "{}/{}",
                    remote,
                    merge.trim_start_matches("refs/heads/")
                ))
            } else {
                None
            }
        } else {
            None
        };

        // Calculate ahead/behind counts if upstream exists
        let (ahead_count, behind_count) = if let Some(ref upstream_ref) = upstream {
            let local_commit_id = match head.peel_to_commit_in_place() {
                Ok(commit) => commit.id().detach(),
                Err(_) => return Ok((upstream, None, None)),
            };

            self.calculate_ahead_behind(repo, local_commit_id, upstream_ref)
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        Ok((upstream, ahead_count, behind_count))
    }

    /// Calculate ahead/behind commit counts between local and upstream
    fn calculate_ahead_behind(
        &self,
        repo: &gix::Repository,
        local_commit_id: gix::ObjectId,
        upstream_ref: &str,
    ) -> Result<(Option<usize>, Option<usize>)> {
        // Convert upstream ref to full path (e.g., "origin/main" -> "refs/remotes/origin/main")
        let upstream_ref_path = if upstream_ref.starts_with("refs/") {
            upstream_ref.to_string()
        } else {
            format!("refs/remotes/{upstream_ref}")
        };

        // Find and peel upstream reference to get commit ID
        let upstream_commit_id = match repo.try_find_reference(&upstream_ref_path) {
            Ok(Some(mut r)) => {
                match r.peel_to_id_in_place() {
                    Ok(id) => id.detach(),
                    Err(_) => return Ok((None, None)),
                }
            }
            Ok(None) | Err(_) => return Ok((None, None)), // Upstream doesn't exist
        };

        // Same commit = no divergence
        if local_commit_id == upstream_commit_id {
            return Ok((Some(0), Some(0)));
        }

        // Find merge base (common ancestor)
        let mut graph = repo.revision_graph(None);
        let merge_base_id = match repo.merge_base_with_graph(
            local_commit_id,
            upstream_commit_id,
            &mut graph,
        ) {
            Ok(base_id) => base_id.detach(),
            Err(_) => return Ok((None, None)), // No common ancestor
        };

        // Count commits between merge base and each branch
        let ahead_count = self.count_commits_between(repo, merge_base_id, local_commit_id)?;
        let behind_count = self.count_commits_between(repo, merge_base_id, upstream_commit_id)?;

        Ok((Some(ahead_count), Some(behind_count)))
    }

    /// Count commits between two points in commit graph
    fn count_commits_between(
        &self,
        repo: &gix::Repository,
        from: gix::ObjectId,
        to: gix::ObjectId,
    ) -> Result<usize> {
        if from == to {
            return Ok(0);
        }

        // Collect all commits reachable from 'from'
        let mut from_commits = std::collections::HashSet::new();
        let from_walker = repo.rev_walk([from])
            .all()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to walk commits from base: {}", e),
            })?;

        for commit_result in from_walker {
            match commit_result {
                Ok(info) => {
                    from_commits.insert(info.id);
                }
                Err(e) => {
                    return Err(crate::error::ReleaseError::Git(GitError::BranchOperationFailed {
                        reason: format!("Error walking commit graph: {}", e),
                    }))
                }
            }
        }

        // Count commits reachable from 'to' that are NOT in from_commits
        let mut count = 0;
        let to_walker = repo.rev_walk([to])
            .all()
            .map_err(|e| GitError::BranchOperationFailed {
                reason: format!("Failed to walk commits to target: {}", e),
            })?;

        for commit_result in to_walker {
            match commit_result {
                Ok(info) => {
                    if !from_commits.contains(&info.id) {
                        count += 1;
                    }
                }
                Err(e) => {
                    return Err(crate::error::ReleaseError::Git(GitError::BranchOperationFailed {
                        reason: format!("Error walking commit graph: {}", e),
                    }))
                }
            }
        }

        Ok(count)
    }

    /// Delete tag from remote using git CLI
    async fn delete_remote_tag_cli(&self, tag_name: &str) -> Result<()> {
        let work_dir = self.work_dir.clone();
        let timeout_duration = tokio::time::Duration::from_secs(300);

        // Build git push command to delete remote tag
        let mut cmd = tokio::process::Command::new("git");
        cmd.current_dir(&work_dir);
        cmd.env("GIT_TERMINAL_PROMPT", "0"); // Prevent credential prompts
        cmd.env("LC_ALL", "C");
        cmd.env("LANG", "C");
        cmd.arg("push");
        cmd.arg("origin"); // Use default remote
        cmd.arg("--delete");
        cmd.arg(format!("refs/tags/{}", tag_name));
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        // Spawn and wait with timeout
        let mut child = cmd.spawn()
            .map_err(|e| GitError::RemoteOperationFailed {
                operation: "delete remote tag".to_string(),
                reason: format!("Failed to spawn git command: {}", e),
            })?;

        let status = tokio::select! {
            result = child.wait() => {
                result.map_err(|e| GitError::RemoteOperationFailed {
                    operation: "delete remote tag".to_string(),
                    reason: format!("Failed to wait for git command: {}", e),
                })?
            }
            () = tokio::time::sleep(timeout_duration) => {
                let _ = child.kill().await;
                return Err(crate::error::ReleaseError::Git(GitError::RemoteOperationFailed {
                    operation: "delete remote tag".to_string(),
                    reason: "Remote tag deletion timed out after 300 seconds".to_string(),
                }));
            }
        };

        // Read stderr for error messages
        if !status.success() {
            let mut stderr_data = Vec::new();
            if let Some(mut stderr) = child.stderr.take() {
                use tokio::io::AsyncReadExt;
                let _ = stderr.read_to_end(&mut stderr_data).await;
            }
            let stderr = String::from_utf8_lossy(&stderr_data);
            return Err(crate::error::ReleaseError::Git(GitError::RemoteOperationFailed {
                operation: "delete remote tag".to_string(),
                reason: format!("Failed to delete remote tag '{}': {}", tag_name, stderr),
            }));
        }

        Ok(())
    }
}