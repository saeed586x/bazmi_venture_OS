//! The central orchestrator of the Venture OS Kernel
use crate::contracts::{
    execution_plan_v1::{
        BackoffStrategy, CompletionCondition, Constraint, ConstraintType, Dependency, Gate,
        GateCriterion, GateType, Goal, ProvenanceInfo, RetryPolicy, Task,
    },
    ExecutionPlanV1, PlanValidationError,
};
use crate::core::{CapabilityRegistry, Governance, Provenance, SemanticModel};
use uuid::Uuid;

/// The central orchestrator that transforms business ideas into executable plans
/// while maintaining strict governance, provenance, and contract adherence.
pub struct Kernel {
    semantic_model: SemanticModel,
    registry: CapabilityRegistry,
    governance: Governance,
    provenance: Provenance,
}

impl Kernel {
    /// Create a new Kernel instance
    pub fn new() -> Self {
        let mut kernel = Self {
            semantic_model: SemanticModel::new(),
            registry: CapabilityRegistry::new(),
            governance: Governance::new(),
            provenance: Provenance::new(),
        };

        // Register default capabilities required by ExecutionPlan.v1
        kernel.register_default_capabilities();

        kernel
    }

    /// Register the default capabilities required for execution plans
    fn register_default_capabilities(&mut self) {
        use crate::core::registry::CapabilityMetadata;

        let default_capabilities = vec![
            CapabilityMetadata {
                name: "intent_engine".to_string(),
                description: "Process and extract structured intents from natural language"
                    .to_string(),
                version: "1.0.0".to_string(),
                interface: "IntentProcessing".to_string(),
                enabled: true,
            },
            CapabilityMetadata {
                name: "planning_engine".to_string(),
                description: "Generate execution plans from intents".to_string(),
                version: "1.0.0".to_string(),
                interface: "Planning".to_string(),
                enabled: true,
            },
            CapabilityMetadata {
                name: "validation_engine".to_string(),
                description: "Validate plans against contracts and policies".to_string(),
                version: "1.0.0".to_string(),
                interface: "Validation".to_string(),
                enabled: true,
            },
            CapabilityMetadata {
                name: "verification_engine".to_string(),
                description: "Verify plan correctness and completeness".to_string(),
                version: "1.0.0".to_string(),
                interface: "Verification".to_string(),
                enabled: true,
            },
        ];

        for cap in default_capabilities {
            self.registry.register_capability(cap);
        }
    }

    /// Process an intent and generate an execution plan
    pub fn process_intent(&self, intent: &str) -> Result<ExecutionPlanV1, KernelError> {
        // Validate intent is not empty or whitespace-only
        let trimmed_intent = intent.trim();
        if trimmed_intent.is_empty() {
            return Err(KernelError::InvalidIntent(
                "Intent cannot be empty or whitespace-only".to_string(),
            ));
        }

        // Generate a unique plan ID as UUID
        let plan_id = Uuid::new_v4().to_string();

        // Generate creation timestamp
        let creation_timestamp = chrono::Utc::now();

        // Create exactly 3 goals as per ExecutionPlan.v1 contract
        let goals = vec![
            Goal {
                id: format!("{}-goal-1", plan_id),
                description: format!("Successfully deliver: {}", trimmed_intent),
                priority: 1,
            },
            Goal {
                id: format!("{}-goal-2", plan_id),
                description: "Ensure all quality gates are passed".to_string(),
                priority: 2,
            },
            Goal {
                id: format!("{}-goal-3", plan_id),
                description: "Maintain compliance with governance policies".to_string(),
                priority: 3,
            },
        ];

        // Create exactly 2 constraints as per ExecutionPlan.v1 contract
        let constraints = vec![
            Constraint {
                id: format!("{}-constraint-1", plan_id),
                description: "Must complete within project timeline".to_string(),
                constraint_type: ConstraintType::Time,
            },
            Constraint {
                id: format!("{}-constraint-2", plan_id),
                description: "Must adhere to security and compliance requirements".to_string(),
                constraint_type: ConstraintType::Compliance,
            },
        ];

        // Create exactly 4 required capabilities as per ExecutionPlan.v1 contract
        let required_capabilities = vec![
            "intent_engine".to_string(),
            "planning_engine".to_string(),
            "validation_engine".to_string(),
            "verification_engine".to_string(),
        ];

        // Verify all required capabilities are registered and enabled
        for cap_name in &required_capabilities {
            match self.registry.get_capability(cap_name) {
                Some(cap) if cap.enabled => {}
                Some(_) => {
                    return Err(KernelError::CapabilityNotFound(format!(
                        "Capability '{}' is not enabled",
                        cap_name
                    )));
                }
                None => {
                    return Err(KernelError::CapabilityNotFound(format!(
                        "Required capability '{}' is not registered",
                        cap_name
                    )));
                }
            }
        }

        // Create exactly 4 tasks as per ExecutionPlan.v1 contract
        let mut task_parameters = std::collections::HashMap::new();
        task_parameters.insert(
            "intent".to_string(),
            serde_json::Value::String(trimmed_intent.to_string()),
        );

        let tasks = vec![
            Task {
                id: format!("{}-task-1", plan_id),
                name: "Intent Analysis".to_string(),
                description: "Analyze and structure the business intent".to_string(),
                capability: "intent_engine".to_string(),
                parameters: task_parameters.clone(),
                expected_duration: Some(300), // 5 minutes
            },
            Task {
                id: format!("{}-task-2", plan_id),
                name: "Plan Generation".to_string(),
                description: "Generate detailed execution plan from analyzed intent".to_string(),
                capability: "planning_engine".to_string(),
                parameters: task_parameters.clone(),
                expected_duration: Some(600), // 10 minutes
            },
            Task {
                id: format!("{}-task-3", plan_id),
                name: "Plan Validation".to_string(),
                description: "Validate generated plan against contracts and policies".to_string(),
                capability: "validation_engine".to_string(),
                parameters: task_parameters.clone(),
                expected_duration: Some(180), // 3 minutes
            },
            Task {
                id: format!("{}-task-4", plan_id),
                name: "Plan Verification".to_string(),
                description: "Verify plan correctness and completeness".to_string(),
                capability: "verification_engine".to_string(),
                parameters: task_parameters,
                expected_duration: Some(180), // 3 minutes
            },
        ];

        // Create at least 1 dependency forming an acyclic DAG
        // Task 2 depends on Task 1, Task 3 depends on Task 2, Task 4 depends on Task 3
        let dependencies = vec![
            Dependency {
                dependent_task_id: format!("{}-task-2", plan_id),
                dependency_task_id: format!("{}-task-1", plan_id),
            },
            Dependency {
                dependent_task_id: format!("{}-task-3", plan_id),
                dependency_task_id: format!("{}-task-2", plan_id),
            },
            Dependency {
                dependent_task_id: format!("{}-task-4", plan_id),
                dependency_task_id: format!("{}-task-3", plan_id),
            },
        ];

        // Validate dependencies form an acyclic DAG
        if !self.validate_dependency_acyclicity(&tasks, &dependencies) {
            return Err(KernelError::InvalidIntent(
                "Task dependencies contain cycles".to_string(),
            ));
        }

        // Create exactly 2 gates as per ExecutionPlan.v1 contract
        let gates = vec![
            Gate {
                id: format!("{}-gate-1", plan_id),
                name: "Quality Gate".to_string(),
                description: "Ensure plan meets quality standards".to_string(),
                gate_type: GateType::Quality,
                criteria: vec![GateCriterion {
                    id: format!("{}-gate-1-criterion-1", plan_id),
                    description: "All tasks have valid capabilities".to_string(),
                    evaluation_method: "capability_validation".to_string(),
                }],
            },
            Gate {
                id: format!("{}-gate-2", plan_id),
                name: "Compliance Gate".to_string(),
                description: "Ensure plan meets compliance requirements".to_string(),
                gate_type: GateType::Compliance,
                criteria: vec![GateCriterion {
                    id: format!("{}-gate-2-criterion-1", plan_id),
                    description: "All governance policies are satisfied".to_string(),
                    evaluation_method: "policy_evaluation".to_string(),
                }],
            },
        ];

        // Create exactly 3 completion conditions as per ExecutionPlan.v1 contract
        let completion_conditions = vec![
            CompletionCondition {
                id: format!("{}-completion-1", plan_id),
                description: "All tasks completed successfully".to_string(),
                expression: "all_tasks_completed".to_string(),
            },
            CompletionCondition {
                id: format!("{}-completion-2", plan_id),
                description: "All gates passed".to_string(),
                expression: "all_gates_passed".to_string(),
            },
            CompletionCondition {
                id: format!("{}-completion-3", plan_id),
                description: "All goals achieved".to_string(),
                expression: "all_goals_achieved".to_string(),
            },
        ];

        // Create provenance information
        let provenance = ProvenanceInfo {
            creator: "venture-os-kernel".to_string(),
            creation_reason: format!("Generated from intent: {}", trimmed_intent),
            evidence_references: vec![format!("intent-hash-{}", Uuid::new_v4())],
        };

        // Build the execution plan
        let plan = ExecutionPlanV1 {
            id: plan_id,
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: trimmed_intent.to_string(),
            goals,
            constraints,
            required_capabilities,
            inputs: vec![],
            tasks,
            dependencies,
            artifacts: vec![],
            gates,
            completion_conditions,
            retry_policy: Some(RetryPolicy {
                max_attempts: 3,
                backoff_strategy: BackoffStrategy::Exponential,
                max_delay_seconds: Some(60),
            }),
            provenance: Some(provenance),
            creation_timestamp,
            replan_reason: None,
        };

        // Validate the plan before returning
        plan.validate()?;

        Ok(plan)
    }

    /// Validate that task dependencies form an acyclic DAG
    fn validate_dependency_acyclicity(&self, tasks: &[Task], dependencies: &[Dependency]) -> bool {
        use std::collections::{HashMap, HashSet};

        // Build adjacency list
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        let task_ids: HashSet<String> = tasks.iter().map(|t| t.id.clone()).collect();

        for dep in dependencies {
            // Verify both task IDs exist
            if !task_ids.contains(&dep.dependent_task_id)
                || !task_ids.contains(&dep.dependency_task_id)
            {
                return false;
            }
            adj.entry(dep.dependency_task_id.as_str())
                .or_default()
                .push(dep.dependent_task_id.as_str());
        }

        // Detect cycles using DFS with explicit lifetime handling
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();

        fn has_cycle(
            node: &str,
            adj: &HashMap<&str, Vec<&str>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
        ) -> bool {
            if rec_stack.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }

            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());

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

        for task_id in &task_ids {
            if !visited.contains(task_id)
                && has_cycle(task_id.as_str(), &adj, &mut visited, &mut rec_stack)
            {
                return false;
            }
        }

        true
    }

    /// Get a reference to the semantic model
    pub fn semantic_model(&self) -> &SemanticModel {
        &self.semantic_model
    }

    /// Get a reference to the capability registry
    pub fn registry(&self) -> &CapabilityRegistry {
        &self.registry
    }

    /// Get a reference to the governance system
    pub fn governance(&self) -> &Governance {
        &self.governance
    }

    /// Get a reference to the provenance system
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum KernelError {
    #[error("Invalid intent: {0}")]
    InvalidIntent(String),
    #[error("Governance violation: {0}")]
    GovernanceViolation(String),
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),
    #[error("Plan validation failed: {0}")]
    PlanValidationFailed(String),
}

impl From<PlanValidationError> for KernelError {
    fn from(err: PlanValidationError) -> Self {
        KernelError::PlanValidationFailed(err.0)
    }
}
