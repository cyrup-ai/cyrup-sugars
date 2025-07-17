// Define traits locally - no external dependencies  
use super::memory_ops::Op;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::memory::{MemoryManager, MemoryNode, MemoryType, Error as MemoryError};

use super::memory_ops;

/// A workflow step that can be stored and executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub description: String,
    pub step_type: StepType,
    pub parameters: serde_json::Value,
    pub dependencies: Vec<String>, // IDs of steps that must complete first
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum StepType {
    Prompt { template: String },
    Transform { function: String },
    Conditional { condition: String, true_branch: String, false_branch: String },
    Parallel { branches: Vec<String> },
    Loop { condition: String, body: String },
}

/// A complete workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub entry_point: String, // ID of the first step
    pub metadata: HashMap<String, serde_json::Value>,
}