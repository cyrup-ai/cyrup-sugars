use std::pin::Pin;
use std::future::Future;
use futures::stream::StreamExt;
// Define Op trait locally - no external dependencies
pub trait Op {
    type Input;
    type Output;
    
    async fn call(&self, input: Self::Input) -> Self::Output;
}
use serde::{Deserialize, Serialize};
use crate::memory::{
    MemoryManager, MemoryNode, MemoryType, MemoryRelationship,
    MemoryMetadata, Error as MemoryError,
};

/// Store a piece of content as a memory node
pub struct StoreMemory<M> {
    manager: M,
    memory_type: MemoryType,
    generate_embedding: bool,
}

impl<M> StoreMemory<M> {
    pub fn new(manager: M, memory_type: MemoryType) -> Self {
        Self {
            manager,
            memory_type,
            generate_embedding: true,
        }
    }

    pub fn without_embedding(mut self) -> Self {
        self.generate_embedding = false;
        self
    }
}

impl<M> Op for StoreMemory<M>
where
    M: MemoryManager + Clone,
{
    type Input = String;
    type Output = Result<MemoryNode, MemoryError>;

    async fn call(&self, input: Self::Input) -> Self::Output {
        let mut memory = MemoryNode::new(input, self.memory_type.clone());
        
        // TODO: Generate embedding if enabled
        if self.generate_embedding {
            // This would integrate with an embedding service
            // For now, we'll use a placeholder
            let embedding = vec![0.1; 768]; // Standard embedding size
            memory = memory.with_embedding(embedding);
        }
        
        self.manager.create_memory(memory).await
    }
}

/// Retrieve memories by vector similarity
pub struct RetrieveMemories<M> {
    manager: M,
    limit: usize,
}

impl<M> RetrieveMemories<M> {
    pub fn new(manager: M, limit: usize) -> Self {
        Self { manager, limit }
    }
}

impl<M> Op for RetrieveMemories<M>
where
    M: MemoryManager + Clone,
{
    type Input = Vec<f32>;
    type Output = Result<Vec<MemoryNode>, MemoryError>;

    async fn call(&self, input: Self::Input) -> Self::Output {
        let mut stream = self.manager.search_by_vector(input, self.limit);
        let mut memories = Vec::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(memory) => memories.push(memory),
                Err(e) => return Err(e),
            }
        }
        
        Ok(memories)
    }
}

/// Search memories by content
pub struct SearchMemories<M> {
    manager: M,
}

impl<M> SearchMemories<M> {
    pub fn new(manager: M) -> Self {
        Self { manager }
    }
}

impl<M> Op for SearchMemories<M>
where
    M: MemoryManager + Clone,
{
    type Input = String;
    type Output = Result<Vec<MemoryNode>, MemoryError>;

    async fn call(&self, input: Self::Input) -> Self::Output {
        let mut stream = self.manager.search_by_content(&input);
        let mut memories = Vec::new();
        
        while let Some(result) = stream.next().await {
            match result {
                Ok(memory) => memories.push(memory),
                Err(e) => return Err(e),
            }
        }
        
        Ok(memories)
    }
}

/// Update memory importance based on access
pub struct UpdateImportance<M> {
    manager: M,
    boost: f32,
}

impl<M> UpdateImportance<M> {
    pub fn new(manager: M, boost: f32) -> Self {
        Self { manager, boost }
    }
}

impl<M> Op for UpdateImportance<M>
where
    M: MemoryManager + Clone,
{
    type Input = MemoryNode;
    type Output = Result<MemoryNode, MemoryError>;

    async fn call(&self, mut input: Self::Input) -> Self::Output {
        // Update importance and last accessed time
        input.metadata.importance = (input.metadata.importance + self.boost).min(1.0);
        input.update_last_accessed();
        
        self.manager.update_memory(input).await
    }
}

/// Create a relationship between two memories
pub struct LinkMemories<M> {
    manager: M,
    relationship_type: String,
}

impl<M> LinkMemories<M> {
    pub fn new(manager: M, relationship_type: String) -> Self {
        Self {
            manager,
            relationship_type,
        }
    }
}

impl<M> Op for LinkMemories<M>
where
    M: MemoryManager + Clone,
{
    type Input = (String, String); // (source_id, target_id)
    type Output = Result<MemoryRelationship, MemoryError>;

    async fn call(&self, input: Self::Input) -> Self::Output {
        let (source_id, target_id) = input;
        let relationship = MemoryRelationship {
            id: uuid::Uuid::new_v4().to_string(),
            source_id,
            target_id,
            relationship_type: self.relationship_type.clone(),
            metadata: None,
        };
        
        self.manager.create_relationship(relationship).await
    }
}

/// Store memory with context - combines storage with relationship creation
pub struct StoreWithContext<M> {
    manager: M,
    memory_type: MemoryType,
}

impl<M> StoreWithContext<M> {
    pub fn new(manager: M, memory_type: MemoryType) -> Self {
        Self { manager, memory_type }
    }
}

impl<M> Op for StoreWithContext<M>
where
    M: MemoryManager + Clone,
{
    type Input = (String, Vec<String>); // (content, related_memory_ids)
    type Output = Result<(MemoryNode, Vec<MemoryRelationship>), MemoryError>;

    async fn call(&self, input: Self::Input) -> Self::Output {
        let (content, related_ids) = input;
        
        // Create the new memory
        let memory = MemoryNode::new(content, self.memory_type.clone());
        let stored_memory = self.manager.create_memory(memory).await?;
        
        // Create relationships to related memories
        let mut relationships = Vec::new();
        for related_id in related_ids {
            let relationship = MemoryRelationship {
                id: uuid::Uuid::new_v4().to_string(),
                source_id: stored_memory.id.clone(),
                target_id: related_id,
                relationship_type: "related_to".to_string(),
                metadata: None,
            };
            
            match self.manager.create_relationship(relationship).await {
                Ok(rel) => relationships.push(rel),
                Err(e) => {
                    // Log error but don't fail the whole operation
                    eprintln!("Failed to create relationship: {}", e);
                }
            }
        }
        
        Ok((stored_memory, relationships))
    }
}

/// Convenience functions for creating memory operations
pub fn store_memory<M: MemoryManager + Clone>(
    manager: M,
    memory_type: MemoryType,
) -> StoreMemory<M> {
    StoreMemory::new(manager, memory_type)
}

pub fn retrieve_memories<M: MemoryManager + Clone>(
    manager: M,
    limit: usize,
) -> RetrieveMemories<M> {
    RetrieveMemories::new(manager, limit)
}

pub fn search_memories<M: MemoryManager + Clone>(manager: M) -> SearchMemories<M> {
    SearchMemories::new(manager)
}

pub fn update_importance<M: MemoryManager + Clone>(
    manager: M,
    boost: f32,
) -> UpdateImportance<M> {
    UpdateImportance::new(manager, boost)
}

pub fn link_memories<M: MemoryManager + Clone>(
    manager: M,
    relationship_type: String,
) -> LinkMemories<M> {
    LinkMemories::new(manager, relationship_type)
}

pub fn store_with_context<M: MemoryManager + Clone>(
    manager: M,
    memory_type: MemoryType,
) -> StoreWithContext<M> {
    StoreWithContext::new(manager, memory_type)
}