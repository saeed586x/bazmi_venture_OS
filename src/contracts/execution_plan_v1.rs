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

impl ExecutionPlanV1 {
    /// Validate the execution plan against the v1 contract
    pub fn validate(&self) -> Result<(), PlanValidationError> {
        // Validate UUID format for id
        if self.id.is_empty() {
            return Err(PlanValidationError("Plan ID cannot be empty".to_string()));
        }

        // Validate semantic version format
        if !self.is_valid_semver(&self.version) {
            return Err(PlanValidationError(format!(
                "Invalid semantic version: {}",
                self.version
            )));
        }

        // Validate required collections are non-empty
        if self.goals.is_empty() {
            return Err(PlanValidationError(
                "Goals collection cannot be empty".to_string(),
            ));
        }
        if self.constraints.is_empty() {
            return Err(PlanValidationError(
                "Constraints collection cannot be empty".to_string(),
            ));
        }
        if self.required_capabilities.is_empty() {
            return Err(PlanValidationError(
                "Required capabilities collection cannot be empty".to_string(),
            ));
        }
        if self.tasks.is_empty() {
            return Err(PlanValidationError(
                "Tasks collection cannot be empty".to_string(),
            ));
        }
        if self.gates.is_empty() {
            return Err(PlanValidationError(
                "Gates collection cannot be empty".to_string(),
            ));
        }
        if self.completion_conditions.is_empty() {
            return Err(PlanValidationError(
                "Completion conditions collection cannot be empty".to_string(),
            ));
        }

        // Validate all task IDs exist for dependencies
        let task_ids: std::collections::HashSet<&str> =
            self.tasks.iter().map(|t| t.id.as_str()).collect();
        for dep in &self.dependencies {
            if !task_ids.contains(dep.dependent_task_id.as_str()) {
                return Err(PlanValidationError(format!(
                    "Dependency references unknown task: {}",
                    dep.dependent_task_id
                )));
            }
            if !task_ids.contains(dep.dependency_task_id.as_str()) {
                return Err(PlanValidationError(format!(
                    "Dependency references unknown task: {}",
                    dep.dependency_task_id
                )));
            }
        }

        // Validate no cyclic dependencies
        if !self.validate_acyclic_dependencies() {
            return Err(PlanValidationError(
                "Task dependencies contain cycles".to_string(),
            ));
        }

        // Validate all gates have non-empty criteria
        for gate in &self.gates {
            if gate.criteria.is_empty() {
                return Err(PlanValidationError(format!(
                    "Gate '{}' has empty criteria",
                    gate.name
                )));
            }
        }

        // Validate provenance is present
        if self.provenance.is_none() {
            return Err(PlanValidationError("Provenance is required".to_string()));
        }

        // Validate replan consistency
        if self.replan_reason.is_some() && self.parent_plan_id.is_none() {
            return Err(PlanValidationError(
                "Replan reason requires parent_plan_id".to_string(),
            ));
        }
        if self.parent_plan_id.is_some() && self.replan_reason.is_none() {
            return Err(PlanValidationError(
                "parent_plan_id requires replan_reason".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if a string is a valid semantic version
    fn is_valid_semver(&self, version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return false;
        }
        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    /// Validate that task dependencies form an acyclic DAG
    fn validate_acyclic_dependencies(&self) -> bool {
        use std::collections::{HashMap, HashSet};

        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        let task_ids: HashSet<&str> = self.tasks.iter().map(|t| t.id.as_str()).collect();

        for dep in &self.dependencies {
            if !task_ids.contains(dep.dependent_task_id.as_str())
                || !task_ids.contains(dep.dependency_task_id.as_str())
            {
                return false;
            }
            adj.entry(dep.dependency_task_id.as_str())
                .or_default()
                .push(dep.dependent_task_id.as_str());
        }

        let mut visited: HashSet<&str> = HashSet::new();
        let mut rec_stack: HashSet<&str> = HashSet::new();

        fn has_cycle<'a>(
            node: &'a str,
            adj: &'a HashMap<&str, Vec<&str>>,
            visited: &mut HashSet<&'a str>,
            rec_stack: &mut HashSet<&'a str>,
        ) -> bool {
            if rec_stack.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }

            visited.insert(node);
            rec_stack.insert(node);

            if let Some(neighbors) = adj.get(node) {
                for &neighbor in neighbors {
                    if has_cycle(neighbor, adj, visited, rec_stack) {
                        return true;
                    }
                }
            }

            rec_stack.remove(node);
            false
        }

        for &task_id in &task_ids {
            if !visited.contains(task_id) && has_cycle(task_id, &adj, &mut visited, &mut rec_stack)
            {
                return false;
            }
        }

        true
    }
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

/// Error type for plan validation failures
#[derive(Debug, thiserror::Error)]
#[error("Plan validation error: {0}")]
pub struct PlanValidationError(pub String);
