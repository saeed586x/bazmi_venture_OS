//! Provenance system for audit trail and evidence tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Provenance system for audit trail and evidence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    /// Audit trail of all actions and decisions
    audit_trail: Vec<AuditEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub target: String,
    pub evidence: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl Provenance {
    /// Create a new provenance system
    pub fn new() -> Self {
        Self {
            audit_trail: vec![],
        }
    }

    /// Record an action in the audit trail
    pub fn record_action(
        &mut self,
        actor: String,
        action: String,
        target: String,
        evidence: Option<String>,
        metadata: Option<serde_json::Value>,
    ) {
        let entry = AuditEntry {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            actor,
            action,
            target,
            evidence,
            metadata,
        };
        self.audit_trail.push(entry);
    }

    /// Get the audit trail
    pub fn audit_trail(&self) -> &[AuditEntry] {
        &self.audit_trail
    }

    /// Get entries for a specific target
    pub fn get_entries_for_target(&self, target: &str) -> Vec<&AuditEntry> {
        self.audit_trail
            .iter()
            .filter(|entry| entry.target == target)
            .collect()
    }
}

impl Default for Provenance {
    fn default() -> Self {
        Self::new()
    }
}
