//! Tool v2 implementation - EXACT API from ARCHITECTURE.md

use serde_json::Value;
use std::marker::PhantomData;
use crate::HashMap;
use crate::domain::agent_role::IntoHashMap;

/// Marker type for Perplexity
pub struct Perplexity;

/// Generic Tool with type parameter
pub struct Tool<T> {
    _phantom: PhantomData<T>,
    config: HashMap<String, Value>,
}

impl<T> Tool<T> {
    /// Create new tool with config - EXACT syntax: Tool<Perplexity>::new({"citations" => "true"})
    pub fn new<C>(config: C) -> Self 
    where
        C: IntoHashMap
    {
        let config_map = config.into_hashmap();
        let mut map = HashMap::new();
        for (k, v) in config_map {
            map.insert(k.to_string(), Value::String(v.to_string()));
        }
        Self {
            _phantom: PhantomData,
            config: map,
        }
    }
}

/// Named tool builder
pub struct NamedTool {
    name: String,
    bin_path: Option<String>,
    description: Option<String>,
}

impl Tool<()> {
    /// Create named tool - EXACT syntax: Tool::named("cargo")
    pub fn named(name: impl Into<String>) -> NamedTool {
        NamedTool {
            name: name.into(),
            bin_path: None,
            description: None,
        }
    }
}

impl NamedTool {
    /// Set binary path - EXACT syntax: .bin("~/.cargo/bin")
    pub fn bin(mut self, path: impl Into<String>) -> Self {
        self.bin_path = Some(path.into());
        self
    }
    
    /// Set description - EXACT syntax: .description("cargo --help".exec_to_text())
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}


// String extension for exec_to_text
pub trait ExecToText {
    fn exec_to_text(&self) -> String;
}

impl ExecToText for &str {
    fn exec_to_text(&self) -> String {
        // Execute command and return output
        std::process::Command::new("sh")
            .arg("-c")
            .arg(self)
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
            .unwrap_or_else(|_| format!("Failed to execute: {}", self))
    }
}

// Implement Send + Sync
unsafe impl<T> Send for Tool<T> {}
unsafe impl<T> Sync for Tool<T> {}
unsafe impl Send for NamedTool {}
unsafe impl Sync for NamedTool {}