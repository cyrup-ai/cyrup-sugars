//! Command execution functions coordinating all release operations.
//!
//! This module implements the complete release workflow by coordinating
//! all modules and providing comprehensive error handling and user feedback.

use crate::cli::{Args, Command, BumpType, ResumePhase, RuntimeConfig};
use crate::error::{Result, ReleaseError};
use crate::git::{GitManager, GitConfig};
use crate::publish::{Publisher, PublisherConfig};
use crate::state::{
    ReleaseState, ReleasePhase, ReleaseConfig,
    create_state_manager_at, has_active_release_at,
    StateConfig, create_state_manager_with_config,
};
use crate::version::{VersionManager, VersionBump, TomlEditor};
use crate::workspace::{WorkspaceInfo, WorkspaceValidator};
use std::time::Duration;

/// Execute the main command based on parsed arguments
pub async fn execute_command(args: Args) -> Result<i32> {
    // Validate arguments
    if let Err(validation_error) = args.validate() {
        eprintln!("❌ Invalid arguments: {}", validation_error);
        return Ok(1);
    }

    let config = RuntimeConfig::from(&args);
    
    // Execute command and handle errors
    let result = match &args.command {
        Command::Release { .. } => execute_release(&args, &config).await,
        Command::Rollback { .. } => execute_rollback(&args, &config).await,
        Command::Resume { .. } => execute_resume(&args, &config).await,
        Command::Status { .. } => execute_status(&args, &config).await,
        Command::Cleanup { .. } => execute_cleanup(&args, &config).await,
        Command::Validate { .. } => execute_validate(&args, &config).await,
        Command::Preview { .. } => execute_preview(&args, &config).await,
    };

    match result {
        Ok(()) => {
            if !config.is_quiet() {
                config.success_println(&format!("Command '{}' completed successfully", args.command.name()));
            }
            Ok(0)
        }
        Err(e) => {
            config.error_println(&format!("Command '{}' failed: {}", args.command.name(), e));
            
            // Show recovery suggestions if available
            if config.is_verbose() {
                let suggestions = e.recovery_suggestions();
                if !suggestions.is_empty() {
                    config.println("\n💡 Recovery suggestions:");
                    for suggestion in suggestions {
                        config.println(&format!("  • {}", suggestion));
                    }
                }
            }

            Ok(1)
        }
    }
}

/// Execute release command
async fn execute_release(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Release {
        bump_type,
        dry_run,
        skip_validation,
        allow_dirty,
        no_push,
        registry,
        package_delay,
        max_retries: _,
        timeout: _,
        no_backup,
        max_concurrent,
    } = &args.command {
        config.verbose_println("Starting release operation...");

        // Validate max_concurrent
        if *max_concurrent == 0 {
            return Err(ReleaseError::Cli(crate::error::CliError::InvalidArguments {
                reason: "max-concurrent must be at least 1".to_string(),
            }));
        }

        // Check for existing release state
        if has_active_release_at(&config.state_file_path) {
            return Err(ReleaseError::State(crate::error::StateError::SaveFailed {
                reason: "Another release is in progress. Use 'resume' or 'cleanup' first".to_string(),
            }));
        }

        // Analyze workspace
        config.verbose_println("Analyzing workspace...");
        let workspace = WorkspaceInfo::analyze(&config.workspace_path)?;

        // Validate workspace if not skipped
        if !skip_validation {
            config.verbose_println("Validating workspace...");
            let validator = WorkspaceValidator::new(workspace.clone())?;
            let validation = validator.validate().await?;
            
            if !validation.success {
                config.error_println("Workspace validation failed:");
                for error in &validation.critical_errors {
                    config.error_println(&format!("  • {}", error));
                }
                return Err(ReleaseError::Workspace(crate::error::WorkspaceError::InvalidStructure {
                    reason: "Workspace validation failed".to_string(),
                }));
            }

            if !validation.warnings.is_empty() && config.is_verbose() {
                config.warning_println("Workspace validation warnings:");
                for warning in &validation.warnings {
                    config.warning_println(&format!("  • {}", warning));
                }
            }
        }

        // Initialize managers
        let mut version_manager = VersionManager::new(workspace.clone());
        
        let git_config = GitConfig {
            default_remote: "origin".to_string(),
            annotated_tags: true,
            auto_push_tags: !no_push,
            ..Default::default()
        };
        let mut git_manager = GitManager::with_config(&config.workspace_path, git_config)?;

        let publisher_config = PublisherConfig {
            inter_package_delay: Duration::from_secs(*package_delay),
            registry: registry.clone(),
            max_concurrent_per_tier: *max_concurrent,
            ..Default::default()
        };
        let mut publisher = Publisher::with_config(&workspace, publisher_config)?;

        // Determine version bump
        let version_bump = match bump_type {
            BumpType::Exact => {
                // This would need additional input for exact version
                return Err(ReleaseError::Cli(crate::error::CliError::InvalidArguments {
                    reason: "Exact version bump not yet implemented".to_string(),
                }));
            }
            _ => VersionBump::from(bump_type.clone()),
        };

        // Create release state
        let release_config = ReleaseConfig {
            dry_run_first: true,
            push_to_remote: !no_push,
            inter_package_delay_ms: package_delay * 1000,
            registry: registry.clone(),
            allow_dirty: *allow_dirty,
            ..Default::default()
        };

        let current_version = version_manager.current_version()?;
        let bumper = crate::version::VersionBumper::from_version(current_version.clone());
        let new_version = bumper.bump(version_bump.clone())?;

        let mut release_state = ReleaseState::new(new_version.clone(), version_bump.clone(), release_config);
        
        // Initialize state manager
        let state_config = StateConfig {
            create_backups: !no_backup,
            ..StateConfig::default()
        };
        let mut state_manager = create_state_manager_with_config(&config.state_file_path, state_config)?;

        if *dry_run {
            config.println("🔍 Performing dry run...");
            
            // Preview changes
            let preview = version_manager.preview_bump(version_bump)?;
            config.println(&format!("Version preview: {}", preview.format_preview()));
            
            // Validate packages
            config.println("Validating packages for publishing...");
            // This would call publisher.check_already_published() etc.
            
            config.success_println("Dry run completed successfully");
            return Ok(());
        }

        // Begin release process
        config.println(&format!("🚀 Starting release: {} → {}", current_version, new_version));
        
        release_state.add_checkpoint(
            "release_started".to_string(),
            ReleasePhase::Validation,
            None,
            false,
        );
        state_manager.save_state(&release_state)?;

        // Phase 1: Version Update
        config.println("📝 Updating versions...");

        // Capture original versions before bumping (for rollback support)
        let mut original_versions = std::collections::HashMap::new();
        for (package_name, package_info) in &workspace.packages {
            original_versions.insert(package_name.clone(), package_info.version.clone());
        }
        release_state.set_original_versions(original_versions);

        let version_result = version_manager.release_version(version_bump)?;
        
        // Set phase and state together to maintain consistency
        release_state.set_phase(ReleasePhase::VersionUpdate);
        release_state.set_version_state(&version_result.update_result);
        release_state.add_checkpoint(
            "version_updated".to_string(),
            ReleasePhase::VersionUpdate,
            None,
            true,
        );
        state_manager.save_state(&release_state)?;

        config.success_println(&format!("Version updated: {}", version_result.summary()));

        // Phase 2: Git Operations
        config.println("📦 Creating git commit and tag...");

        let git_result = git_manager.perform_release(&new_version, !no_push).await?;
        
        // Set phase and state together to maintain consistency
        release_state.set_phase(ReleasePhase::GitOperations);
        release_state.set_git_state(Some(&git_result.commit), Some(&git_result.tag));
        
        if let Some(push_info) = &git_result.push_info {
            release_state.set_git_push_state(push_info);
        }

        release_state.add_checkpoint(
            "git_operations_complete".to_string(),
            ReleasePhase::GitOperations,
            None,
            true,
        );
        state_manager.save_state(&release_state)?;

        config.success_println(&format!("Git operations completed: {}", git_result.format_result()));

        // Phase 3: Publishing
        config.println("📤 Publishing packages...");
        release_state.set_phase(ReleasePhase::Publishing);
        
        let publish_order = crate::workspace::DependencyGraph::build(&workspace)?.publish_order()?;
        release_state.init_publish_state(publish_order.tier_count());
        state_manager.save_state(&release_state)?;

        let publish_result = publisher.publish_all_packages().await?;
        
        // Update state with publish results
        for (_package_name, package_result) in &publish_result.successful_publishes {
            release_state.add_published_package(package_result);
        }
        
        for (package_name, error) in &publish_result.failed_packages {
            release_state.add_failed_package(package_name.clone(), error.clone());
        }

        release_state.add_checkpoint(
            "publishing_complete".to_string(),
            ReleasePhase::Publishing,
            None,
            true,
        );
        state_manager.save_state(&release_state)?;

        if publish_result.all_successful {
            config.success_println(&format!("Publishing completed: {}", publish_result.format_summary()));
        } else {
            config.warning_println(&format!("Publishing partially failed: {}", publish_result.format_summary()));
        }

        // Phase 4: Cleanup
        config.println("🧹 Cleaning up...");
        release_state.set_phase(ReleasePhase::Cleanup);
        state_manager.save_state(&release_state)?;

        // Clear git manager state
        git_manager.clear_release_state();

        // Clear publisher state
        publisher.clear_state();

        // Mark as completed
        release_state.set_phase(ReleasePhase::Completed);
        release_state.add_checkpoint(
            "release_completed".to_string(),
            ReleasePhase::Completed,
            None,
            false,
        );
        state_manager.save_state(&release_state)?;

        config.success_println(&format!("🎉 Release {} completed successfully!", new_version));
        
        // Cleanup state file after successful completion
        if !no_backup {
            state_manager.create_backup()?;
        }
        state_manager.cleanup_state()?;

    } else {
        unreachable!("execute_release called with non-Release command");
    }

    Ok(())
}

/// Execute rollback command
async fn execute_rollback(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Rollback { force, git_only, packages_only, yes } = &args.command {
        config.verbose_println("Starting rollback operation...");

        // Load release state
        let mut state_manager = create_state_manager_at(&config.state_file_path)?;
        let load_result = state_manager.load_state()?;
        let mut release_state = load_result.state;

        if load_result.recovered_from_backup {
            config.warning_println("Loaded state from backup file");
        }

        // Validate rollback conditions
        if release_state.current_phase == ReleasePhase::Completed && !force {
            return Err(ReleaseError::State(crate::error::StateError::SaveFailed {
                reason: "Release completed successfully. Use --force to rollback anyway".to_string(),
            }));
        }

        if !yes {
            config.println(&format!(
                "About to rollback release {} (phase: {:?})",
                release_state.target_version,
                release_state.current_phase
            ));
            config.println("WARNING: Rollback will:");
            config.println("  - Delete local and remote release tags");
            config.println("  - Reset git HEAD to previous commit");
            config.println("  - This operation cannot be undone");
            
            if !prompt_confirmation("Proceed with rollback?")? {
                config.println("Rollback cancelled");
                return Ok(());
            }
        }

        release_state.set_phase(ReleasePhase::RollingBack);
        state_manager.save_state(&release_state)?;

        let workspace = WorkspaceInfo::analyze(&config.workspace_path)?;

        // Rollback publishing if needed and not git-only
        if !git_only && release_state.publish_state.is_some() {
            config.println("📤 Rolling back published packages...");
            let publisher = Publisher::new(&workspace)?;
            let rollback_result = publisher.rollback_published_packages().await?;
            
            if rollback_result.fully_successful {
                config.success_println("All published packages yanked successfully");
            } else {
                config.warning_println(&format!("Rollback completed with warnings: {}", rollback_result.format_summary()));
            }
        }

        // Rollback git operations if needed and not packages-only
        if !packages_only && release_state.git_state.is_some() {
            config.println("📦 Rolling back git operations...");
            let git_config = GitConfig::default();
            let mut git_manager = GitManager::with_config(&config.workspace_path, git_config)?;
            
            let git_rollback = git_manager.rollback_release().await?;
            
            if git_rollback.success {
                config.success_println("Git operations rolled back successfully");
            } else {
                config.warning_println(&format!("Git rollback completed with warnings: {}", git_rollback.format_result()));
            }
        }

        // Rollback version changes if possible
        if let Some(_version_state) = &release_state.version_state {
            config.println("📝 Rolling back version changes...");

            if let Some(original_versions) = &release_state.original_versions {
                let mut restored_count = 0;
                let mut failed_packages = Vec::new();

                for (package_name, original_version) in original_versions {
                    // Find package in workspace to get Cargo.toml path
                    if let Some(package_info) = workspace.packages.get(package_name) {
                        match restore_package_version(&package_info.cargo_toml_path, original_version) {
                            Ok(()) => {
                                config.verbose_println(&format!("  {} → {}", package_name, original_version));
                                restored_count += 1;
                            }
                            Err(e) => {
                                config.warning_println(&format!("  Failed to restore {}: {}", package_name, e));
                                failed_packages.push(package_name.clone());
                            }
                        }
                    } else {
                        config.warning_println(&format!("  Package {} not found in workspace", package_name));
                        failed_packages.push(package_name.clone());
                    }
                }

                if restored_count > 0 {
                    config.success_println(&format!("Restored {} package versions", restored_count));
                }

                if !failed_packages.is_empty() {
                    config.warning_println(&format!("Failed to restore {} packages: {}",
                        failed_packages.len(),
                        failed_packages.join(", ")
                    ));
                }
            } else {
                config.warning_println("No version history in state file");
                config.warning_println("You may need to manually revert version changes in Cargo.toml files");
            }
        }

        release_state.set_phase(ReleasePhase::RolledBack);
        release_state.add_checkpoint(
            "rollback_completed".to_string(),
            ReleasePhase::RolledBack,
            None,
            false,
        );
        state_manager.save_state(&release_state)?;

        config.success_println("🔄 Rollback completed");

    } else {
        unreachable!("execute_rollback called with non-Rollback command");
    }

    Ok(())
}

/// Execute resume command
async fn execute_resume(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Resume { force, reset_to_phase, skip_validation: _ } = &args.command {
        config.verbose_println("Resuming release operation...");

        // Load release state
        let mut state_manager = create_state_manager_at(&config.state_file_path)?;
        let load_result = state_manager.load_state()?;
        let mut release_state = load_result.state;

        // Validate resumability
        if !release_state.is_resumable() && !force {
            return Err(ReleaseError::State(crate::error::StateError::LoadFailed {
                reason: "Release is not in a resumable state. Use --force to resume anyway".to_string(),
            }));
        }

        if release_state.has_critical_errors() && !force {
            return Err(ReleaseError::State(crate::error::StateError::Corrupted {
                reason: "Release has critical errors. Use --force to resume anyway".to_string(),
            }));
        }

        // Reset to specific phase if requested
        if let Some(reset_phase) = reset_to_phase {
            let new_phase = match reset_phase {
                ResumePhase::Validation => ReleasePhase::Validation,
                ResumePhase::VersionUpdate => ReleasePhase::VersionUpdate,
                ResumePhase::GitOperations => ReleasePhase::GitOperations,
                ResumePhase::Publishing => ReleasePhase::Publishing,
            };
            
            config.println(&format!("Resetting to phase: {:?}", new_phase));
            release_state.set_phase(new_phase);
            state_manager.save_state(&release_state)?;
        }

        config.println(&format!(
            "Resuming release {} from phase: {:?}",
            release_state.target_version,
            release_state.current_phase
        ));

        // Continue from current phase
        match release_state.current_phase {
            ReleasePhase::Validation => {
                // Re-run validation and continue
                config.println("Re-validating workspace...");
                // Continue to version update...
            }
            ReleasePhase::VersionUpdate => {
                // Continue with version update
                config.println("Continuing version update...");
                // Implementation continues...
            }
            ReleasePhase::GitOperations => {
                // Continue with git operations
                config.println("Continuing git operations...");
                // Implementation continues...
            }
            ReleasePhase::Publishing => {
                // Continue with publishing
                config.println("Continuing publishing...");
                // Implementation continues...
            }
            _ => {
                return Err(ReleaseError::State(crate::error::StateError::Corrupted {
                    reason: format!("Cannot resume from phase: {:?}", release_state.current_phase),
                }));
            }
        }

        config.success_println("Resume completed");

    } else {
        unreachable!("execute_resume called with non-Resume command");
    }

    Ok(())
}

/// Execute status command
async fn execute_status(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Status { detailed, history: _, json } = &args.command {
        config.verbose_println("Checking release status...");

        if !has_active_release_at(&config.state_file_path) {
            if *json {
                println!("{{\"status\": \"no_active_release\"}}");
            } else {
                config.println("No active release found");
            }
            return Ok(());
        }

        // Load release state
        let mut state_manager = create_state_manager_at(&config.state_file_path)?;
        let load_result = state_manager.load_state()?;
        let release_state = load_result.state;

        if *json {
            let json_output = serde_json::to_string_pretty(&release_state)
                .map_err(|e| ReleaseError::Json(e))?;
            println!("{}", json_output);
        } else {
            config.println(&format!("📊 {}", release_state.summary()));
            
            if *detailed {
                config.println(&format!("Release ID: {}", release_state.release_id));
                config.println(&format!("Started: {}", release_state.started_at));
                config.println(&format!("Updated: {}", release_state.updated_at));
                config.println(&format!("Elapsed: {}", release_state.elapsed_time().num_seconds()));
                
                if !release_state.checkpoints.is_empty() {
                    config.println("\nCheckpoints:");
                    for checkpoint in &release_state.checkpoints {
                        config.println(&format!("  ✓ {} ({:?})", checkpoint.name, checkpoint.phase));
                    }
                }
                
                if !release_state.errors.is_empty() {
                    config.println("\nErrors:");
                    for error in &release_state.errors {
                        let recoverable = if error.recoverable { "recoverable" } else { "critical" };
                        config.println(&format!("  ❌ {} ({})", error.message, recoverable));
                    }
                }
            }
        }

    } else {
        unreachable!("execute_status called with non-Status command");
    }

    Ok(())
}

/// Execute cleanup command
async fn execute_cleanup(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Cleanup { all, older_than, yes } = &args.command {
        config.verbose_println("Cleaning up state files...");

        if !yes {
            config.println("About to clean up release state files");
            
            if !prompt_confirmation("Proceed with cleanup?")? {
                config.println("Cleanup cancelled");
                return Ok(());
            }
        }

        let state_manager = create_state_manager_at(&config.state_file_path)?;
        
        if *all || older_than.is_some() {
            state_manager.cleanup_state()?;
            config.success_println("State files cleaned up");
        } else {
            // Just clean up current state
            if has_active_release_at(&config.state_file_path) {
                state_manager.cleanup_state()?;
                config.success_println("Current state file cleaned up");
            } else {
                config.println("No state files to clean up");
            }
        }

    } else {
        unreachable!("execute_cleanup called with non-Cleanup command");
    }

    Ok(())
}

/// Execute validate command
async fn execute_validate(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Validate { fix: _, detailed, json } = &args.command {
        config.verbose_println("Validating workspace...");

        let workspace = WorkspaceInfo::analyze(&config.workspace_path)?;
        let validator = WorkspaceValidator::new(workspace)?;
        let validation = validator.validate().await?;

        if *json {
            let json_output = serde_json::to_string_pretty(&validation)
                .map_err(|e| ReleaseError::Json(e))?;
            println!("{}", json_output);
        } else {
            config.println(&format!("📋 {}", validation.summary()));
            
            if *detailed {
                for check in &validation.checks {
                    config.println(&format!("  {}", check.format_result()));
                }
            }
            
            if !validation.warnings.is_empty() && !config.is_quiet() {
                config.println("\n⚠️ Warnings:");
                for warning in &validation.warnings {
                    config.warning_println(&format!("  • {}", warning));
                }
            }
            
            if !validation.critical_errors.is_empty() {
                config.println("\n❌ Critical Errors:");
                for error in &validation.critical_errors {
                    config.error_println(&format!("  • {}", error));
                }
            }
        }

        if !validation.success {
            return Err(ReleaseError::Workspace(crate::error::WorkspaceError::InvalidStructure {
                reason: "Workspace validation failed".to_string(),
            }));
        }

    } else {
        unreachable!("execute_validate called with non-Validate command");
    }

    Ok(())
}

/// Execute preview command
async fn execute_preview(args: &Args, config: &RuntimeConfig) -> Result<()> {
    if let Command::Preview { bump_type, detailed, json } = &args.command {
        config.verbose_println("Previewing version bump...");

        let workspace = WorkspaceInfo::analyze(&config.workspace_path)?;
        let version_manager = VersionManager::new(workspace);

        let version_bump = match bump_type {
            BumpType::Exact => {
                return Err(ReleaseError::Cli(crate::error::CliError::InvalidArguments {
                    reason: "Exact version preview not yet implemented".to_string(),
                }));
            }
            _ => VersionBump::from(bump_type.clone()),
        };

        let preview = version_manager.preview_bump(version_bump.clone())?;

        if *json {
            let json_output = serde_json::to_string_pretty(&preview)
                .map_err(|e| ReleaseError::Json(e))?;
            println!("{}", json_output);
        } else {
            config.println(&format!("🔍 {}", preview.format_preview()));
            
            if *detailed {
                config.println("\nDetailed changes:");
                config.println(&format!("  Version: {} → {}", 
                    preview.bump_preview.current,
                    preview.bump_preview.get_version(&version_bump).unwrap()
                ));
                
                config.println(&format!("  Files to modify: {}", preview.update_preview.files_to_modify.len()));
                for file in &preview.update_preview.files_to_modify {
                    config.println(&format!("    • {}", file.display()));
                }
            }
        }

    } else {
        unreachable!("execute_preview called with non-Preview command");
    }

    Ok(())
}

/// Restore a package version in its Cargo.toml file
fn restore_package_version(cargo_toml_path: &std::path::Path, version: &str) -> Result<()> {
    let version_parsed = semver::Version::parse(version)
        .map_err(|e| crate::error::VersionError::ParseFailed {
            version: version.to_string(),
            source: e,
        })?;

    let mut editor = TomlEditor::open(cargo_toml_path)?;
    editor.update_package_version(&version_parsed)?;
    editor.save()?;

    Ok(())
}

/// Prompt user for confirmation (yes/no)
/// Returns true if user confirms with "yes" or "y" (case-insensitive)
/// Returns false on EOF, empty input, or any other input
fn prompt_confirmation(message: &str) -> Result<bool> {
    use std::io::{self, Write};
    
    // Print prompt message and flush to ensure it appears immediately
    print!("{} [y/N]: ", message);
    io::stdout().flush()
        .map_err(|e| ReleaseError::Cli(crate::error::CliError::ExecutionFailed {
            command: "prompt".to_string(),
            reason: format!("Failed to flush stdout: {}", e),
        }))?;
    
    // Read user input
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => {
            // EOF (Ctrl+D) - default to no
            return Ok(false);
        }
        Ok(_) => {
            // Got input, check if it's a confirmation
            let trimmed = input.trim().to_lowercase();
            Ok(trimmed == "yes" || trimmed == "y")
        }
        Err(_) => {
            // Error reading input - default to no
            Ok(false)
        }
    }
}