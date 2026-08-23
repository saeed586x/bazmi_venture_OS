//! The central orchestrator of the Venture OS Kernel
use crate::contracts::ExecutionPlanV1;
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
        Self {
            semantic_model: SemanticModel::new(),
            registry: CapabilityRegistry::new(),
            governance: Governance::new(),
            provenance: Provenance::new(),
        }
    }

    /// Process an intent and generate an execution plan
    pub fn process_intent(&self, intent: &str) -> Result<ExecutionPlanV1, KernelError> {
        // Deterministic plan generation based on intent analysis
        // This ensures contract compliance without requiring real LLM calls

        let intent_lower = intent.to_lowercase();

        // Extract key themes from intent for deterministic generation
        let (domain_keyword, action_keyword) = Self::extract_intent_keywords(&intent_lower);

        // Generate deterministic IDs based on intent hash
        let intent_hash = Self::hash_intent(intent);

        // Build goals (minimum 1 required)
        let goals = vec![crate::contracts::Goal {
            id: format!("goal-{:x}-1", intent_hash),
            description: format!("Successfully {} {} system", action_keyword, domain_keyword),
            priority: 1,
        }];

        // Build constraints (at least 1 for completeness)
        let constraints = vec![crate::contracts::Constraint {
            id: format!("constraint-{:x}-1", intent_hash),
            description: format!("Must comply with {} domain standards", domain_keyword),
            constraint_type: crate::contracts::ConstraintType::Compliance,
        }];

        // Build required capabilities (minimum 1 required)
        let required_capabilities = vec![
            "planning".to_string(),
            "execution".to_string(),
            "validation".to_string(),
        ];

        // Build inputs
        let inputs = vec![crate::contracts::Input {
            id: format!("input-{:x}-1", intent_hash),
            name: "intent".to_string(),
            data_type: "string".to_string(),
            default_value: Some(serde_json::Value::String(intent.to_string())),
        }];

        // Build tasks (minimum 3 required per contract)
        let mut tasks = Vec::new();
        tasks.push(crate::contracts::Task {
            id: format!("task-{:x}-1", intent_hash),
            name: format!("Analyze {} requirements", domain_keyword),
            description: format!(
                "Parse and analyze requirements for {} {}",
                action_keyword, domain_keyword
            ),
            capability: "planning".to_string(),
            parameters: [(
                "intent".to_string(),
                serde_json::Value::String(intent.to_string()),
            )]
            .iter()
            .cloned()
            .collect(),
            expected_duration: Some(60),
        });
        tasks.push(crate::contracts::Task {
            id: format!("task-{:x}-2", intent_hash),
            name: format!("Design {} architecture", domain_keyword),
            description: format!(
                "Create architectural design for {} {}",
                action_keyword, domain_keyword
            ),
            capability: "planning".to_string(),
            parameters: [(
                "domain".to_string(),
                serde_json::Value::String(domain_keyword.clone()),
            )]
            .iter()
            .cloned()
            .collect(),
            expected_duration: Some(120),
        });
        tasks.push(crate::contracts::Task {
            id: format!("task-{:x}-3", intent_hash),
            name: format!("Implement {} solution", domain_keyword),
            description: format!(
                "Implement the {} {} solution",
                action_keyword, domain_keyword
            ),
            capability: "execution".to_string(),
            parameters: [(
                "action".to_string(),
                serde_json::Value::String(action_keyword.clone()),
            )]
            .iter()
            .cloned()
            .collect(),
            expected_duration: Some(300),
        });

        // Build dependencies (minimum 1 required)
        let dependencies = vec![
            crate::contracts::Dependency {
                dependent_task_id: format!("task-{:x}-2", intent_hash),
                dependency_task_id: format!("task-{:x}-1", intent_hash),
            },
            crate::contracts::Dependency {
                dependent_task_id: format!("task-{:x}-3", intent_hash),
                dependency_task_id: format!("task-{:x}-2", intent_hash),
            },
        ];

        // Build artifacts
        let artifacts = vec![crate::contracts::Artifact {
            id: format!("artifact-{:x}-1", intent_hash),
            name: format!("{}_requirements", domain_keyword),
            artifact_type: "document".to_string(),
            location: format!("/artifacts/{}/requirements.md", domain_keyword),
        }];

        // Build gates (minimum 1 required)
        let gates = vec![crate::contracts::Gate {
            id: format!("gate-{:x}-1", intent_hash),
            name: "Quality Gate".to_string(),
            description: format!("Ensure {} meets quality standards", domain_keyword),
            gate_type: crate::contracts::GateType::Quality,
            criteria: vec![crate::contracts::GateCriterion {
                id: format!("criterion-{:x}-1", intent_hash),
                description: "All tests pass".to_string(),
                evaluation_method: "automated".to_string(),
            }],
        }];

        // Build completion conditions (minimum 1 required)
        let completion_conditions = vec![crate::contracts::CompletionCondition {
            id: format!("completion-{:x}-1", intent_hash),
            description: format!("{} system successfully delivered", domain_keyword),
            expression: format!("all_tasks_complete AND quality_gate_passed"),
        }];

        Ok(ExecutionPlanV1 {
            id: Uuid::new_v4().to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: intent.to_string(),
            goals,
            constraints,
            required_capabilities,
            inputs,
            tasks,
            dependencies,
            artifacts,
            gates,
            completion_conditions,
            retry_policy: Some(crate::contracts::RetryPolicy {
                max_attempts: 3,
                backoff_strategy: crate::contracts::BackoffStrategy::Exponential,
                max_delay_seconds: Some(300),
            }),
            provenance: Some(crate::contracts::ProvenanceInfo {
                creator: "kernel".to_string(),
                creation_reason: "intent_processing".to_string(),
                evidence_references: vec![intent.to_string()],
            }),
            creation_timestamp: chrono::Utc::now(),
            replan_reason: None,
        })
    }

    /// Extract deterministic keywords from intent for plan generation
    fn extract_intent_keywords(intent: &str) -> (String, String) {
        // Deterministic keyword extraction based on common patterns
        let domain_keywords = [
            "customer",
            "portal",
            "system",
            "app",
            "service",
            "platform",
            "api",
            "dashboard",
            "tool",
            "module",
        ];
        let action_keywords = [
            "build",
            "create",
            "develop",
            "implement",
            "design",
            "deploy",
            "setup",
            "configure",
        ];

        let domain = domain_keywords
            .iter()
            .find(|&&k| intent.contains(k))
            .unwrap_or(&"system")
            .to_string();

        let action = action_keywords
            .iter()
            .find(|&&k| intent.contains(k))
            .unwrap_or(&"build")
            .to_string();

        (domain, action)
    }

    /// Generate a deterministic hash from intent for ID generation
    fn hash_intent(intent: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        intent.hash(&mut hasher);
        hasher.finish()
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
