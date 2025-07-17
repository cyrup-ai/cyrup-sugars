// Define traits locally - no external dependencies
pub trait Prompt: Clone {
    async fn prompt(&self, input: String) -> Result<String, PromptError>;
}

#[derive(Debug, thiserror::Error)]
pub enum PromptError {
    #[error("Prompt error: {0}")]
    Error(String),
}

// Import Op from memory_ops
use super::memory_ops::Op;
use crate::memory::{MemoryManager, MemoryNode, MemoryType, Error as MemoryError};

use super::memory_ops::{self, StoreMemory, RetrieveMemories, SearchMemories};

// Workflow builder module
mod workflow {
    use super::*;
    
    pub fn new() -> WorkflowBuilder {
        WorkflowBuilder
    }
    
    pub struct WorkflowBuilder;
    
    impl WorkflowBuilder {
        pub fn chain<O>(self, op: O) -> O {
            op
        }
    }
}

// For now, let's simplify and not use parallel macro
fn passthrough<T: Clone + Send + Sync + 'static>() -> impl Op<Input = T, Output = T> {
    struct PassthroughOp<T> {
        _phantom: std::marker::PhantomData<T>,
    }
    
    impl<T: Clone + Send + Sync + 'static> Op for PassthroughOp<T> {
        type Input = T;
        type Output = T;
        
        async fn call(&self, input: Self::Input) -> Self::Output {
            input
        }
    }
    
    PassthroughOp { _phantom: std::marker::PhantomData }
}

// Tuple operation that runs two ops and returns a tuple
fn run_both<I, O1, O2, Op1, Op2>(op1: Op1, op2: Op2) -> impl Op<Input = I, Output = (O1, O2)>
where
    I: Clone + Send + Sync + 'static,
    O1: Send + Sync + 'static,
    O2: Send + Sync + 'static,
    Op1: Op<Input = I, Output = O1>,
    Op2: Op<Input = I, Output = O2>,
{
    struct BothOp<Op1, Op2> {
        op1: Op1,
        op2: Op2,
    }
    
    impl<I, O1, O2, Op1, Op2> Op for BothOp<Op1, Op2>
    where
        I: Clone + Send + Sync + 'static,
        O1: Send + Sync + 'static,
        O2: Send + Sync + 'static,
        Op1: Op<Input = I, Output = O1>,
        Op2: Op<Input = I, Output = O2>,
    {
        type Input = I;
        type Output = (O1, O2);
        
        async fn call(&self, input: Self::Input) -> Self::Output {
            let result1 = self.op1.call(input.clone()).await;
            let result2 = self.op2.call(input).await;
            (result1, result2)
        }
    }
    
    BothOp { op1, op2 }
}

// Production memory workflow implementation
struct MemoryWorkflowOp<M, P> {
    memory_manager: M,
    prompt_model: P,
    context_limit: usize,
}

impl<M, P> Op for MemoryWorkflowOp<M, P>
where
    M: MemoryManager + Clone,
    P: Prompt,
{
    type Input = String;
    type Output = Result<String, WorkflowError>;

    async fn call(&self, input: Self::Input) -> Self::Output {
        // Store input as episodic memory
        let _ = memory_ops::store_memory(self.memory_manager.clone(), MemoryType::Episodic)
            .call(input.clone())
            .await;

        // Search for relevant memories
        let memories = memory_ops::search_memories(self.memory_manager.clone())
            .call(input.clone())
            .await
            .unwrap_or_default();

        // Format the prompt with context
        let context = memories
            .iter()
            .take(self.context_limit)
            .map(|m| format!("- {}", m.content))
            .collect::<Vec<_>>()
            .join("\n");
        
        let formatted_input = if context.is_empty() {
            input
        } else {
            format!("Previous context:\n{}\n\nCurrent query: {}", context, input)
        };

        // Prompt the model
        let response = self.prompt_model
            .prompt(formatted_input)
            .await
            .map_err(WorkflowError::Prompt)?;

        // Store response as semantic memory
        let _ = memory_ops::store_memory(self.memory_manager.clone(), MemoryType::Semantic)
            .call(response.clone())
            .await;

        Ok(response)
    }
}

/// A memory-enhanced workflow that stores inputs, retrieves context, and generates responses
pub struct MemoryEnhancedWorkflow<M, P> {
    memory_manager: M,
    prompt_model: P,
    context_limit: usize,
}

impl<M, P> MemoryEnhancedWorkflow<M, P>
where
    M: MemoryManager + Clone,
    P: Prompt,
{
    pub fn new(memory_manager: M, prompt_model: P) -> Self {
        Self {
            memory_manager,
            prompt_model,
            context_limit: 5,
        }
    }

    pub fn with_context_limit(mut self, limit: usize) -> Self {
        self.context_limit = limit;
        self
    }

    /// Build the memory-enhanced workflow
    pub fn build(self) -> impl Op<Input = String, Output = Result<String, WorkflowError>> {
        let memory_manager = self.memory_manager;
        let prompt_model = self.prompt_model;
        let context_limit = self.context_limit;

        // Return a simple Op implementation
        MemoryWorkflowOp {
            memory_manager,
            prompt_model,
            context_limit,
        }
    }
}

/// Error type for memory workflows
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Memory error: {0}")]
    Memory(#[from] MemoryError),
    
    #[error("Prompt error: {0}")]
    Prompt(#[from] PromptError),
    
    #[error("Workflow error: {0}")]
    Other(String),
}

/// Create a simple memory-aware conversation workflow
pub fn conversation_workflow<M, P>(
    memory_manager: M,
    prompt_model: P,
) -> impl Op<Input = String, Output = Result<String, WorkflowError>>
where
    M: MemoryManager + Clone,
    P: Prompt,
{
    MemoryEnhancedWorkflow::new(memory_manager, prompt_model)
        .with_context_limit(10)
        .build()
}

/// Create a learning workflow that adapts based on feedback
pub struct AdaptiveWorkflow<M, B> {
    memory_manager: M,
    base_op: B,
}

impl<M, B> AdaptiveWorkflow<M, B>
where
    M: MemoryManager + Clone,
    B: Op,
{
    pub fn new(memory_manager: M, base_op: B) -> Self {
        Self {
            memory_manager,
            base_op,
        }
    }
}

impl<M, B> Op for AdaptiveWorkflow<M, B>
where
    M: MemoryManager + Clone,
    B: Op,
    B::Input: Clone + serde::Serialize,
    B::Output: Clone + serde::Serialize,
{
    type Input = B::Input;
    type Output = (B::Output, String); // (result, memory_id)

    async fn call(&self, input: Self::Input) -> Self::Output {
        // Execute the base operation
        let output = self.base_op.call(input.clone()).await;
        
        // Create a memory capturing both input and output
        let memory_content = serde_json::json!({
            "input": input,
            "output": output,
            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        }).to_string();
        
        let memory = MemoryNode::new(memory_content, MemoryType::Episodic)
            .with_importance(0.5); // Initial neutral importance
        
        let stored_memory = self.memory_manager
            .create_memory(memory)
            .await
            .expect("Failed to store memory");
        
        (output, stored_memory.id)
    }
}

/// Apply feedback to a stored memory
pub async fn apply_feedback<M: MemoryManager>(
    memory_manager: M,
    memory_id: String,
    feedback: f32,
) -> Result<(), MemoryError> {
    if let Some(mut memory) = memory_manager.get_memory(&memory_id).await? {
        // Update importance based on feedback
        memory.metadata.importance = feedback.clamp(0.0, 1.0);
        memory_manager.update_memory(memory).await?;
    }
    Ok(())
}

/// Create a RAG (Retrieval-Augmented Generation) workflow
pub fn rag_workflow<M, P>(
    memory_manager: M,
    prompt_model: P,
    retrieval_limit: usize,
) -> impl Op<Input = String, Output = Result<String, WorkflowError>>
where
    M: MemoryManager + Clone,
    P: Prompt,
{
    RagWorkflowOp {
        memory_manager,
        prompt_model,
        retrieval_limit,
    }
}

struct RagWorkflowOp<M, P> {
    memory_manager: M,
    prompt_model: P,
    retrieval_limit: usize,
}

impl<M, P> Op for RagWorkflowOp<M, P>
where
    M: MemoryManager + Clone,
    P: Prompt,
{
    type Input = String;
    type Output = Result<String, WorkflowError>;

    async fn call(&self, query: Self::Input) -> Self::Output {
        // Retrieve relevant documents
        let memories = memory_ops::search_memories(self.memory_manager.clone())
            .call(query.clone())
            .await
            .unwrap_or_default();
        
        // Format as RAG prompt
        let documents = memories
            .iter()
            .take(self.retrieval_limit)
            .enumerate()
            .map(|(i, m)| format!("Document {}: {}", i + 1, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        
        let prompt = format!(
            "Using the following documents as context:\n\n{}\n\nAnswer the question: {}",
            documents, query
        );

        // Generate response
        self.prompt_model
            .prompt(prompt)
            .await
            .map_err(WorkflowError::Prompt)
    }
}