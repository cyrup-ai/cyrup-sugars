//! Comprehensive error types for cyrup_release operations.
//!
//! This module defines all error types with actionable error messages and recovery suggestions.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for cyrup_release operations
pub type Result<T> = std::result::Result<T, ReleaseError>;

/// Main error type for all cyrup_release operations
#[derive(Error, Debug)]
pub enum ReleaseError {
    /// Workspace analysis errors
    #[error("Workspace error: {0}")]
    Workspace(#[from] WorkspaceError),

    /// Version management errors
    #[error("Version error: {0}")]
    Version(#[from] VersionError),

    /// Git operation errors
    #[error("Git error: {0}")]
    Git(#[from] GitError),

    /// Publishing errors
    #[error("Publish error: {0}")]
    Publish(#[from] PublishError),

    /// State management errors
    #[error("State error: {0}")]
    State(#[from] StateError),

    /// CLI argument errors
    #[error("CLI error: {0}")]
    Cli(#[from] CliError),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing errors
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// TOML editing errors
    #[error("TOML edit error: {0}")]
    TomlEdit(#[from] toml_edit::TomlError),
}

/// Workspace-specific errors
#[derive(Error, Debug)]
pub enum WorkspaceError {
    /// Workspace root not found
    #[error("Could not find workspace root. Please run from within a Cargo workspace.")]
    RootNotFound,

    /// Invalid workspace structure
    #[error("Invalid workspace structure: {reason}")]
    InvalidStructure {
        /// Reason for the invalid structure
        reason: String
    },

    /// Package not found in workspace
    #[error("Package '{name}' not found in workspace")]
    PackageNotFound {
        /// Name of the package that was not found
        name: String
    },

    /// Circular dependency detected
    #[error("Circular dependency detected in packages: {packages:?}")]
    CircularDependency {
        /// List of packages involved in the circular dependency
        packages: Vec<String>
    },

    /// Missing Cargo.toml file
    #[error("Missing Cargo.toml file at {path}")]
    MissingCargoToml {
        /// Path where Cargo.toml was expected
        path: PathBuf
    },

    /// Invalid package configuration
    #[error("Invalid package configuration for '{package}': {reason}")]
    InvalidPackage {
        /// Name of the package with invalid configuration
        package: String,
        /// Reason for the invalid configuration
        reason: String
    },
}

/// Version management errors
#[derive(Error, Debug)]
pub enum VersionError {
    /// Invalid version format
    #[error("Invalid version '{version}': {reason}")]
    InvalidVersion {
        /// The invalid version string
        version: String,
        /// Reason why the version is invalid
        reason: String
    },

    /// Version parsing failed
    #[error("Failed to parse version '{version}': {source}")]
    ParseFailed {
        /// The version string that failed to parse
        version: String,
        /// The underlying semver parsing error
        #[source]
        source: semver::Error,
    },

    /// Internal dependency version mismatch
    #[error("Internal dependency version mismatch for '{dependency}': expected {expected}, found {found}")]
    DependencyMismatch {
        /// Name of the dependency with version mismatch
        dependency: String,
        /// Expected version string
        expected: String,
        /// Actual version found
        found: String,
    },

    /// Failed to update Cargo.toml
    #[error("Failed to update Cargo.toml at {path}: {reason}")]
    TomlUpdateFailed {
        /// Path to the Cargo.toml file
        path: PathBuf,
        /// Reason for the update failure
        reason: String
    },

    /// Version bump not supported
    #[error("Version bump '{bump}' not supported for version '{version}'")]
    UnsupportedBump {
        /// The bump type that was requested
        bump: String,
        /// The current version
        version: String
    },
}

/// Git operation errors
#[derive(Error, Debug)]
pub enum GitError {
    /// Not a git repository
    #[error("Not a git repository. Please initialize git first.")]
    NotRepository,

    /// Working directory not clean
    #[error("Working directory not clean. Please commit or stash changes before releasing.")]
    DirtyWorkingDirectory,

    /// Git authentication failed
    #[error("Git authentication failed: {reason}")]
    AuthenticationFailed {
        /// Reason for authentication failure
        reason: String
    },

    /// Remote operation failed
    #[error("Git remote operation failed: {operation} - {reason}")]
    RemoteOperationFailed {
        /// The git operation that failed
        operation: String,
        /// Reason for the failure
        reason: String
    },

    /// Tag already exists
    #[error("Git tag '{tag}' already exists. Use --force to overwrite or choose a different version.")]
    TagExists {
        /// The tag name that already exists
        tag: String
    },

    /// Branch operation failed
    #[error("Git branch operation failed: {reason}")]
    BranchOperationFailed {
        /// Reason for branch operation failure
        reason: String
    },

    /// Commit failed
    #[error("Git commit failed: {reason}")]
    CommitFailed {
        /// Reason for commit failure
        reason: String
    },

    /// Push failed
    #[error("Git push failed: {reason}")]
    PushFailed {
        /// Reason for push failure
        reason: String
    },
}

/// Publishing errors
#[derive(Error, Debug)]
pub enum PublishError {
    /// Package already published
    #[error("Package '{package}' version '{version}' already published to crates.io")]
    AlreadyPublished {
        /// Name of the package
        package: String,
        /// Version that is already published
        version: String
    },

    /// Publish command failed
    #[error("Cargo publish failed for '{package}': {reason}")]
    PublishFailed {
        /// Name of the package that failed to publish
        package: String,
        /// Reason for the publish failure
        reason: String
    },

    /// Dry run validation failed
    #[error("Dry run validation failed for '{package}': {reason}")]
    DryRunFailed {
        /// Name of the package that failed dry run
        package: String,
        /// Reason for the dry run failure
        reason: String
    },

    /// Rate limit exceeded
    #[error("Rate limit exceeded for crates.io. Please wait {retry_after_seconds} seconds before retrying.")]
    RateLimitExceeded {
        /// Number of seconds to wait before retrying
        retry_after_seconds: u64
    },

    /// Network error during publishing
    #[error("Network error during publishing: {reason}")]
    NetworkError {
        /// Reason for the network error
        reason: String
    },

    /// Authentication error for crates.io
    #[error("Authentication error: Please ensure you're logged in with 'cargo login'")]
    AuthenticationError,

    /// Yank operation failed
    #[error("Failed to yank package '{package}' version '{version}': {reason}")]
    YankFailed {
        /// Name of the package to yank
        package: String,
        /// Version to yank
        version: String,
        /// Reason for the yank failure
        reason: String,
    },
}

/// State management errors
#[derive(Error, Debug)]
pub enum StateError {
    /// State file corrupted
    #[error("State file corrupted: {reason}")]
    Corrupted {
        /// Reason for state corruption
        reason: String
    },

    /// State file not found
    #[error("State file not found. No release in progress.")]
    NotFound,

    /// State version mismatch
    #[error("State file version mismatch: expected {expected}, found {found}")]
    VersionMismatch {
        /// Expected state file version
        expected: String,
        /// Actual state file version found
        found: String
    },

    /// Failed to save state
    #[error("Failed to save state: {reason}")]
    SaveFailed {
        /// Reason for save failure
        reason: String
    },

    /// Failed to load state
    #[error("Failed to load state: {reason}")]
    LoadFailed {
        /// Reason for load failure
        reason: String
    },
}

/// CLI-specific errors
#[derive(Error, Debug)]
pub enum CliError {
    /// Invalid command line arguments
    #[error("Invalid arguments: {reason}")]
    InvalidArguments {
        /// Reason for invalid arguments
        reason: String
    },

    /// Missing required argument
    #[error("Missing required argument: {argument}")]
    MissingArgument {
        /// Name of the missing argument
        argument: String
    },

    /// Conflicting arguments
    #[error("Conflicting arguments: {arguments:?}")]
    ConflictingArguments {
        /// List of conflicting argument names
        arguments: Vec<String>
    },

    /// Command execution failed
    #[error("Command execution failed: {command} - {reason}")]
    ExecutionFailed {
        /// The command that failed
        command: String,
        /// Reason for execution failure
        reason: String
    },
}

impl ReleaseError {
    /// Get actionable recovery suggestions for this error
    pub fn recovery_suggestions(&self) -> Vec<String> {
        match self {
            ReleaseError::Workspace(WorkspaceError::RootNotFound) => vec![
                "Navigate to a directory containing a Cargo workspace".to_string(),
                "Ensure you have a Cargo.toml file with [workspace] section".to_string(),
            ],
            ReleaseError::Workspace(WorkspaceError::CircularDependency { packages }) => vec![
                format!("Review dependencies between packages: {}", packages.join(", ")),
                "Remove circular dependencies by restructuring package relationships".to_string(),
            ],
            ReleaseError::Git(GitError::DirtyWorkingDirectory) => vec![
                "Commit pending changes: git add . && git commit -m 'message'".to_string(),
                "Stash changes temporarily: git stash".to_string(),
                "Reset working directory: git reset --hard HEAD".to_string(),
            ],
            ReleaseError::Git(GitError::AuthenticationFailed { .. }) => vec![
                "Check SSH key configuration: ssh -T git@github.com".to_string(),
                "Verify git remote URL: git remote -v".to_string(),
                "Regenerate SSH keys if needed".to_string(),
            ],
            ReleaseError::Publish(PublishError::AuthenticationError) => vec![
                "Login to crates.io: cargo login".to_string(),
                "Verify API token is valid and has publish permissions".to_string(),
            ],
            ReleaseError::Publish(PublishError::RateLimitExceeded { retry_after_seconds }) => vec![
                format!("Wait {} seconds before retrying", retry_after_seconds),
                "Use --publish-interval to add delays between packages".to_string(),
            ],
            _ => vec!["Check the error message above for specific details".to_string()],
        }
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        match self {
            ReleaseError::Workspace(WorkspaceError::RootNotFound) => false,
            ReleaseError::Workspace(WorkspaceError::CircularDependency { .. }) => false,
            ReleaseError::Git(GitError::NotRepository) => false,
            ReleaseError::Version(VersionError::InvalidVersion { .. }) => false,
            ReleaseError::Publish(PublishError::AlreadyPublished { .. }) => false,
            _ => true,
        }
    }
}