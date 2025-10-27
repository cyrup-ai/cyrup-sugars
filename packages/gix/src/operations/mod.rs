//! Git repository operations using gix (Gitoxide)

pub mod commit;

// Re-export commit operation
pub use commit::{CommitOpts, Signature, commit};
