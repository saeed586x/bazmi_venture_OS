//! The canonical semantic model - the single source of truth for all domain concepts

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The canonical semantic model that serves as the single source of truth
/// for all domain concepts in the Venture OS Kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticModel {
    /// Registry of domain entities and their relationships
    entities: HashMap<String, DomainEntity>,
    /// Version information for the semantic model
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub attributes: HashMap<String, String>,
    pub relationships: Vec<String>,
}

impl SemanticModel {
    /// Create a new semantic model
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            version: "1.0.0".to_string(),
        }
    }

    /// Register a new domain entity
    pub fn register_entity(&mut self, entity: DomainEntity) {
        self.entities.insert(entity.id.clone(), entity);
    }

    /// Get a domain entity by ID
    pub fn get_entity(&self, id: &str) -> Option<&DomainEntity> {
        self.entities.get(id)
    }

    /// Get all domain entities
    pub fn entities(&self) -> &HashMap<String, DomainEntity> {
        &self.entities
    }

    /// Get the semantic model version
    pub fn version(&self) -> &str {
        &self.version
    }
}

impl Default for SemanticModel {
    fn default() -> Self {
        Self::new()
    }
}
