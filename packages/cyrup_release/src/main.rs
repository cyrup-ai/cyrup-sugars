//! Cyrup Release - Production-quality release management for Rust workspaces.
//!
//! This binary provides atomic release operations with proper error handling,
//! automatic internal dependency version synchronization, and rollback capabilities.

use cyrup_release::cli;
use std::process;

#[tokio::main]
async fn main() {
    // Set up proper error handling and graceful shutdown
    let result = cli::run().await;

    match result {
        Ok(exit_code) => {
            process::exit(exit_code);
        }
        Err(e) => {
            eprintln!("❌ Fatal error: {}", e);
            
            // Show recovery suggestions for critical errors
            let suggestions = e.recovery_suggestions();
            if !suggestions.is_empty() {
                eprintln!("\n💡 Recovery suggestions:");
                for suggestion in suggestions {
                    eprintln!("  • {}", suggestion);
                }
            }
            
            process::exit(1);
        }
    }
}