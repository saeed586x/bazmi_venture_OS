//! The central orchestrator of the Venture OS Kernel
use crate::contracts::ExecutionPlanV1;
use crate::core::{CapabilityRegistry, Governance, Provenance, SemanticModel};

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
        Self {
            semantic_model: SemanticModel::new(),
            registry: CapabilityRegistry::new(),
            governance: Governance::new(),
            provenance: Provenance::new(),
        }
    }

    /// Process an intent and generate an execution plan
    pub fn process_intent(&self, intent: &str) -> Result<ExecutionPlanV1, KernelError> {
        // Validate intent
        if intent.trim().is_empty() {
            return Err(KernelError::InvalidIntent(
                "Intent cannot be empty".to_string(),
            ));
        }

        // Use the Intent Engine to process the intent
        let intent_engine = crate::capabilities::intent_engine::IntentEngine::default();
        let processed_intent = intent_engine
            .process_intent(intent)
            .map_err(|e| KernelError::InvalidIntent(e.to_string()))?;

        // Create initial execution plan from intent
        let mut plan = intent_engine.create_execution_plan(&processed_intent);

        // Generate meaningful goals from the intent
        plan.goals = vec![
            crate::contracts::execution_plan_v1::Goal {
                id: uuid::Uuid::new_v4().to_string(),
                description: format!("Successfully implement: {}", intent),
                priority: 1,
            },
            crate::contracts::execution_plan_v1::Goal {
                id: uuid::Uuid::new_v4().to_string(),
                description: "Validate solution meets requirements".to_string(),
                priority: 2,
            },
            crate::contracts::execution_plan_v1::Goal {
                id: uuid::Uuid::new_v4().to_string(),
                description: "Deploy and monitor solution".to_string(),
                priority: 3,
            },
        ];

        // Add constraints
        plan.constraints = vec![
            crate::contracts::execution_plan_v1::Constraint {
                id: uuid::Uuid::new_v4().to_string(),
                description: "Must follow security best practices".to_string(),
                constraint_type: crate::contracts::execution_plan_v1::ConstraintType::Compliance,
            },
            crate::contracts::execution_plan_v1::Constraint {
                id: uuid::Uuid::new_v4().to_string(),
                description: "Must be scalable".to_string(),
                constraint_type: crate::contracts::execution_plan_v1::ConstraintType::Resource,
            },
        ];

        // Add required capabilities
        plan.required_capabilities = vec![
            "domain-modeling".to_string(),
            "requirements-analysis".to_string(),
            "planning".to_string(),
            "validation".to_string(),
        ];

        // Generate tasks based on the intent
        plan.tasks = vec![
            crate::contracts::execution_plan_v1::Task {
                id: "task-1".to_string(),
                name: "Analyze Requirements".to_string(),
                description: format!("Analyze requirements for: {}", intent),
                capability: "requirements-analysis".to_string(),
                parameters: serde_json::Map::new().into_iter().collect(),
                expected_duration: Some(7200), // 2 hours
            },
            crate::contracts::execution_plan_v1::Task {
                id: "task-2".to_string(),
                name: "Design Solution".to_string(),
                description: "Design architecture and components".to_string(),
                capability: "domain-modeling".to_string(),
                parameters: serde_json::Map::new().into_iter().collect(),
                expected_duration: Some(14400), // 4 hours
            },
            crate::contracts::execution_plan_v1::Task {
                id: "task-3".to_string(),
                name: "Implement Core Features".to_string(),
                description: "Implement the core functionality".to_string(),
                capability: "planning".to_string(),
                parameters: serde_json::Map::new().into_iter().collect(),
                expected_duration: Some(28800), // 8 hours
            },
            crate::contracts::execution_plan_v1::Task {
                id: "task-4".to_string(),
                name: "Validate and Test".to_string(),
                description: "Validate solution against requirements".to_string(),
                capability: "validation".to_string(),
                parameters: serde_json::Map::new().into_iter().collect(),
                expected_duration: Some(7200), // 2 hours
            },
        ];

        // Create dependencies between tasks
        plan.dependencies = vec![
            crate::contracts::execution_plan_v1::Dependency {
                dependent_task_id: "task-2".to_string(),
                dependency_task_id: "task-1".to_string(),
            },
            crate::contracts::execution_plan_v1::Dependency {
                dependent_task_id: "task-3".to_string(),
                dependency_task_id: "task-2".to_string(),
            },
            crate::contracts::execution_plan_v1::Dependency {
                dependent_task_id: "task-4".to_string(),
                dependency_task_id: "task-3".to_string(),
            },
        ];

        // Add gates
        plan.gates = vec![
            crate::contracts::execution_plan_v1::Gate {
                id: "gate-1".to_string(),
                name: "Requirements Review".to_string(),
                description: "Review and approve requirements".to_string(),
                gate_type: crate::contracts::execution_plan_v1::GateType::Quality,
                criteria: vec![crate::contracts::execution_plan_v1::GateCriterion {
                    id: "criterion-1".to_string(),
                    description: "All requirements are clear and testable".to_string(),
                    evaluation_method: "Manual Review".to_string(),
                }],
            },
            crate::contracts::execution_plan_v1::Gate {
                id: "gate-2".to_string(),
                name: "Architecture Review".to_string(),
                description: "Review and approve architecture design".to_string(),
                gate_type: crate::contracts::execution_plan_v1::GateType::Quality,
                criteria: vec![crate::contracts::execution_plan_v1::GateCriterion {
                    id: "criterion-2".to_string(),
                    description: "Architecture follows best practices".to_string(),
                    evaluation_method: "Peer Review".to_string(),
                }],
            },
        ];

        // Add completion conditions
        plan.completion_conditions = vec![
            crate::contracts::execution_plan_v1::CompletionCondition {
                id: "completion-1".to_string(),
                description: "All tasks completed successfully".to_string(),
                expression: "all_tasks_completed == true".to_string(),
            },
            crate::contracts::execution_plan_v1::CompletionCondition {
                id: "completion-2".to_string(),
                description: "All gates passed".to_string(),
                expression: "all_gates_passed == true".to_string(),
            },
            crate::contracts::execution_plan_v1::CompletionCondition {
                id: "completion-3".to_string(),
                description: "All goals achieved".to_string(),
                expression: "all_goals_achieved == true".to_string(),
            },
        ];

        // Add provenance
        plan.provenance = Some(crate::contracts::execution_plan_v1::ProvenanceInfo {
            creator: "venture-os-kernel".to_string(),
            creation_reason: "Initial plan generation from intent".to_string(),
            evidence_references: vec![processed_intent.original_text.clone()],
        });

        Ok(plan)
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
}
