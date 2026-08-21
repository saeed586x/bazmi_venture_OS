//! XState adapter for state machine visualization

use crate::contracts::ExecutionPlanV1;
use serde::{Deserialize, Serialize};

/// XState adapter for state machine visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStateAdapter {
    /// Configuration for the XState adapter
    config: XStateConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStateConfig {
    pub enable_visualization: bool,
    pub export_format: ExportFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Scxml,
    Mermaid,
}

impl XStateAdapter {
    /// Create a new XState adapter
    pub fn new(config: XStateConfig) -> Self {
        Self { config }
    }

    /// Convert an execution plan to XState format
    pub fn to_xstate(&self, plan: &ExecutionPlanV1) -> Result<XStateMachine, XStateError> {
        // In a real implementation, this would convert the execution plan
        // to an XState-compatible state machine representation

        Ok(XStateMachine {
            id: format!("plan_{}", plan.id),
            initial: "start".to_string(),
            states: vec![],
            transitions: vec![],
        })
    }

    /// Export XState machine to specified format
    pub fn export(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        match self.config.export_format {
            ExportFormat::Json => Ok(serde_json::to_string(machine)?),
            ExportFormat::Scxml => Ok("SCXML export not implemented".to_string()),
            ExportFormat::Mermaid => Ok("Mermaid export not implemented".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStateMachine {
    pub id: String,
    pub initial: String,
    pub states: Vec<XState>,
    pub transitions: Vec<XTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XState {
    pub id: String,
    pub name: String,
    pub type_: StateType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateType {
    Atomic,
    Compound,
    Parallel,
    Final,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XTransition {
    pub from: String,
    pub to: String,
    pub event: String,
    pub guard: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum XStateError {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Export not supported: {0}")]
    UnsupportedExport(String),
}
