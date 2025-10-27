//! Command line interface for cyrup_release.
//!
//! This module provides a comprehensive CLI for release management operations,
//! with proper argument parsing, command execution, and user feedback.

mod args;
mod commands;

pub use args::{
    Args, Command, BumpType, ResumePhase, RuntimeConfig, VerbosityLevel,
};
pub use commands::execute_command;

use crate::error::Result;

/// Main CLI entry point
pub async fn run() -> Result<i32> {
    let args = Args::parse_args();
    execute_command(args).await
}

/// Parse arguments without executing (for testing)
pub fn parse_args() -> Args {
    Args::parse_args()
}

/// Validate arguments without executing (for testing)
pub fn validate_args(args: &Args) -> crate::error::Result<()> {
    args.validate().map_err(|e| crate::error::CliError::InvalidArguments { reason: e }.into())
}

/// Create runtime configuration from arguments
pub fn create_runtime_config(args: &Args) -> RuntimeConfig {
    RuntimeConfig::from(args)
}