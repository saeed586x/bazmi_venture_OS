//! ExecutionPlan.v1 contract - the standardized format for executable plans

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ExecutionPlan.v1 - Standardized format for executable plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlanV1 {
    /// Unique identifier for this execution plan
    pub id: String,

    /// Version of this execution plan
    pub version: String,

    /// Reference to a parent plan if this is a replan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_plan_id: Option<String>,

    /// Reference to the original intent that led to this plan
    pub intent_reference: String,

    /// Goals to be achieved by this plan
    pub goals: Vec<Goal>,

    /// Constraints that must be respected during execution
    pub constraints: Vec<Constraint>,

    /// Capabilities required to execute this plan
    pub required_capabilities: Vec<String>,

    /// Input data required for plan execution
    pub inputs: Vec<Input>,

    /// Tasks to be executed as part of this plan
    pub tasks: Vec<Task>,

    /// Dependencies between tasks
    pub dependencies: Vec<Dependency>,

    /// Artifacts produced by this plan
    pub artifacts: Vec<Artifact>,

    /// Gates that must be passed during execution
    pub gates: Vec<Gate>,

    /// Conditions that determine when the plan is complete
    pub completion_conditions: Vec<CompletionCondition>,

    /// Retry policy for failed tasks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,

    /// Provenance information for this plan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ProvenanceInfo>,

    /// Timestamp when this plan was created
    pub creation_timestamp: DateTime<Utc>,

    /// Reason for replanning if this is a replan
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replan_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub description: String,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub id: String,
    pub description: String,
    pub constraint_type: ConstraintType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Time,
    Resource,
    Compliance,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub id: String,
    pub name: String,
    pub data_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub description: String,
    pub capability: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub expected_duration: Option<u64>, // in seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub dependent_task_id: String,
    pub dependency_task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub artifact_type: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub gate_type: GateType,
    pub criteria: Vec<GateCriterion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GateType {
    Quality,
    Security,
    Compliance,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateCriterion {
    pub id: String,
    pub description: String,
    pub evaluation_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionCondition {
    pub id: String,
    pub description: String,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub max_delay_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Linear,
    Exponential,
    Fixed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvenanceInfo {
    pub creator: String,
    pub creation_reason: String,
    pub evidence_references: Vec<String>,
}
