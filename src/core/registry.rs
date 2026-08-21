//! Capability registry for discovering and managing kernel capabilities

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Registry for discovering and managing kernel capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRegistry {
    /// Map of capability names to their metadata
    capabilities: HashMap<String, CapabilityMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub interface: String,
    pub enabled: bool,
}

impl CapabilityRegistry {
    /// Create a new capability registry
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    /// Register a new capability
    pub fn register_capability(&mut self, metadata: CapabilityMetadata) {
        self.capabilities.insert(metadata.name.clone(), metadata);
    }

    /// Get a capability by name
    pub fn get_capability(&self, name: &str) -> Option<&CapabilityMetadata> {
        self.capabilities.get(name)
    }

    /// Get all capabilities
    pub fn capabilities(&self) -> &HashMap<String, CapabilityMetadata> {
        &self.capabilities
    }

    /// Check if a capability is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.capabilities
            .get(name)
            .map(|cap| cap.enabled)
            .unwrap_or(false)
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
