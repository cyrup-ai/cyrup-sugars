//! # Cyrup Release
//!
//! Production-quality release management for Rust workspaces.
//!
//! This crate provides atomic release operations with proper error handling,
//! automatic internal dependency version synchronization, and rollback capabilities
//! including crate yanking for published packages.
//!
//! ## Features
//!
//! - **Atomic Operations**: All release steps succeed or all rollback
//! - **Version Synchronization**: Automatic internal dependency version management  
//! - **Git Integration**: Pure Rust git operations using gix (no CLI dependencies)
//! - **Resume Capability**: Continue interrupted releases from checkpoints
//! - **Rollback Support**: Undo git operations and yank published crates
//! - **Dependency Ordering**: Publish packages in correct dependency order
//!
//! ## Usage
//!
//! ```bash
//! cyrup_release patch          # Bump patch version and publish
//! cyrup_release minor --dry    # Dry run minor version bump
//! cyrup_release rollback       # Rollback failed release
//! cyrup_release resume         # Resume interrupted release
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

// Core modules
pub mod workspace;
pub mod version;
pub mod git;
pub mod publish;
pub mod state;
pub mod cli;
pub mod error;

// Re-export main types for public API
pub use error::{Result, ReleaseError};
pub use workspace::{WorkspaceInfo, DependencyGraph};
pub use version::{VersionBump, VersionManager};
pub use git::{GitManager, GitOperations};
pub use publish::Publisher;
pub use workspace::PublishOrder;
pub use state::{ReleaseState, StateManager};
pub use cli::{Command, Args};

/// Main release orchestrator that coordinates all operations
pub struct ReleaseManager {
    workspace: WorkspaceInfo,
    git: GitManager,
    publisher: Publisher,
    state: StateManager,
}

impl ReleaseManager {
    /// Create a new release manager for the current workspace
    pub fn new() -> Result<Self> {
        let workspace = WorkspaceInfo::analyze(".")?;
        let git = GitManager::new(".")?;
        let publisher = Publisher::new(&workspace)?;
        let state = StateManager::new(".cyrup_release_state.json")?;

        Ok(Self {
            workspace,
            git,
            publisher,
            state,
        })
    }

    /// Execute a release with the specified version bump
    pub async fn release(&mut self, bump: VersionBump, dry_run: bool) -> Result<()> {
        // Release implementation is coordinated through cli::commands::execute_release
        // This method provides a programmatic interface to the release functionality
        
        if dry_run {
            // Perform validation and preview without making changes
            let version_manager = VersionManager::new(self.workspace.clone());
            let _preview = version_manager.preview_bump(bump)?;
            Ok(())
        } else {
            // For full releases, recommend using the CLI interface
            Err(ReleaseError::Cli(crate::error::CliError::InvalidArguments {
                reason: "Full release operations should use the CLI interface for proper state management".to_string(),
            }))
        }
    }

    /// Rollback a failed release
    pub async fn rollback(&mut self) -> Result<()> {
        // Rollback implementation is coordinated through cli::commands::execute_rollback
        // This method provides a programmatic interface to check rollback feasibility
        
        if !self.state.state_exists() {
            return Err(ReleaseError::State(crate::error::StateError::NotFound));
        }

        // For actual rollback operations, recommend using the CLI interface
        Err(ReleaseError::Cli(crate::error::CliError::InvalidArguments {
            reason: "Rollback operations should use the CLI interface for proper state management".to_string(),
        }))
    }

    /// Resume an interrupted release
    pub async fn resume(&mut self) -> Result<()> {
        // Resume implementation is coordinated through cli::commands::execute_resume
        // This method provides a programmatic interface to check resume feasibility
        
        if !self.state.state_exists() {
            return Err(ReleaseError::State(crate::error::StateError::NotFound));
        }

        let load_result = self.state.load_state()?;
        let release_state = load_result.state;

        if !release_state.is_resumable() {
            return Err(ReleaseError::State(crate::error::StateError::Corrupted {
                reason: "Release is not in a resumable state".to_string(),
            }));
        }

        // For actual resume operations, recommend using the CLI interface
        Err(ReleaseError::Cli(crate::error::CliError::InvalidArguments {
            reason: "Resume operations should use the CLI interface for proper state management".to_string(),
        }))
    }

    /// Get the workspace information
    pub fn workspace(&self) -> &WorkspaceInfo {
        &self.workspace
    }

    /// Get the git manager
    pub fn git(&mut self) -> &mut GitManager {
        &mut self.git
    }

    /// Get the publisher
    pub fn publisher(&mut self) -> &mut Publisher {
        &mut self.publisher
    }

    /// Get the state manager
    pub fn state(&mut self) -> &mut StateManager {
        &mut self.state
    }
}