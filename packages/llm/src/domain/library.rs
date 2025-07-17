//! Library type for memory system - EXACT API from ARCHITECTURE.md

/// Library type for memory
pub struct Library {
    name: String,
}

impl Library {
    /// Create a named library - EXACT syntax: Library::named("obsidian_vault")
    pub fn named(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
        }
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }
}

