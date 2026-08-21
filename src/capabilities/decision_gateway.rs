//! Decision Gateway - controls and routes decision-making processes

use crate::capabilities::risk_engine::{RiskAssessment, RiskEngine};
use crate::capabilities::validation_engine::{ValidationEngine, ValidationResult};
use crate::capabilities::verification_engine::{VerificationEngine, VerificationResult};
use crate::contracts::ExecutionPlanV1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Decision Gateway - controls and routes decision-making processes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionGateway {
    /// Decision routing rules
    routing_rules: HashMap<String, DecisionRoute>,
    /// Decision policies
    policies: HashMap<String, DecisionPolicy>,
    /// References to capability engines
    validation_engine: ValidationEngine,
    verification_engine: VerificationEngine,
    risk_engine: RiskEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRoute {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: RoutingCondition,
    pub target_engine: DecisionTarget,
    pub priority: RoutePriority,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingCondition {
    PlanComplexity { min_tasks: usize, max_tasks: usize },
    RiskLevel { min_level: RiskLevelFilter },
    ValidationStatus { required_score: f64 },
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevelFilter {
    Low,
    Medium,
    High,
    Critical,
    Any,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionTarget {
    Validation,
    Verification,
    RiskAssessment,
    Planning,
    Approval,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutePriority {
    Low,
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionPolicy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub policy_type: PolicyType,
    pub rules: Vec<PolicyRule>,
    pub enforcement_level: EnforcementLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    Approval,
    Validation,
    Routing,
    Escalation,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: String,
    pub description: String,
    pub condition: String,
    pub action: PolicyAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    Escalate { to_role: String },
    Delegate { to_engine: String },
    Conditional { conditions: Vec<String> },
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnforcementLevel {
    Advisory,
    Mandatory,
    Blocking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionContext {
    pub plan: ExecutionPlanV1,
    pub validation_result: Option<ValidationResult>,
    pub verification_result: Option<VerificationResult>,
    pub risk_assessment: Option<RiskAssessment>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionOutcome {
    pub decision_id: String,
    pub decision_type: DecisionType,
    pub outcome: DecisionResult,
    pub reasoning: String,
    pub next_steps: Vec<NextStep>,
    pub confidence: f64, // 0.0 to 1.0
    pub made_at: chrono::DateTime<chrono::Utc>,
    pub made_by: DecisionMaker,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionType {
    PlanApproval,
    RiskMitigation,
    ResourceAllocation,
    TimelineAdjustment,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionResult {
    Approved,
    Rejected,
    Conditional { conditions: Vec<String> },
    Deferred { reason: String },
    Escalated { to: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextStep {
    pub id: String,
    pub description: String,
    pub assignee: String,
    pub due_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DecisionMaker {
    Automated,
    Human { role: String, name: Option<String> },
    Hybrid { automated_confidence_threshold: f64 },
}

impl DecisionGateway {
    /// Create a new decision gateway
    pub fn new(
        validation_engine: ValidationEngine,
        verification_engine: VerificationEngine,
        risk_engine: RiskEngine,
    ) -> Self {
        let mut gateway = Self {
            routing_rules: HashMap::new(),
            policies: HashMap::new(),
            validation_engine,
            verification_engine,
            risk_engine,
        };

        // Set up default routing rules
        gateway.setup_default_routing_rules();
        gateway.setup_default_policies();

        gateway
    }

    /// Set up default routing rules
    fn setup_default_routing_rules(&mut self) {
        // Route complex plans to verification
        self.routing_rules.insert(
            "complex-plans".to_string(),
            DecisionRoute {
                id: "complex-plans".to_string(),
                name: "Complex Plans".to_string(),
                description: "Route plans with many tasks to verification".to_string(),
                condition: RoutingCondition::PlanComplexity {
                    min_tasks: 10,
                    max_tasks: usize::MAX,
                },
                target_engine: DecisionTarget::Verification,
                priority: RoutePriority::Normal,
            },
        );

        // Route high-risk plans to risk assessment
        self.routing_rules.insert(
            "high-risk".to_string(),
            DecisionRoute {
                id: "high-risk".to_string(),
                name: "High Risk Plans".to_string(),
                description: "Route high-risk plans to risk assessment".to_string(),
                condition: RoutingCondition::RiskLevel {
                    min_level: RiskLevelFilter::High,
                },
                target_engine: DecisionTarget::RiskAssessment,
                priority: RoutePriority::High,
            },
        );

        // Route all plans to validation
        self.routing_rules.insert(
            "validation".to_string(),
            DecisionRoute {
                id: "validation".to_string(),
                name: "Validation Route".to_string(),
                description: "All plans go through validation".to_string(),
                condition: RoutingCondition::Custom("always".to_string()),
                target_engine: DecisionTarget::Validation,
                priority: RoutePriority::Critical,
            },
        );
    }

    /// Set up default policies
    fn setup_default_policies(&mut self) {
        // Approval policy based on validation score
        self.policies.insert(
            "approval-policy".to_string(),
            DecisionPolicy {
                id: "approval-policy".to_string(),
                name: "Plan Approval Policy".to_string(),
                description: "Policy for approving execution plans".to_string(),
                policy_type: PolicyType::Approval,
                rules: vec![
                    PolicyRule {
                        id: "validation-threshold".to_string(),
                        description: "Plans must have validation score above 0.8".to_string(),
                        condition: "validation_score > 0.8".to_string(),
                        action: PolicyAction::Allow,
                    },
                    PolicyRule {
                        id: "critical-risk-block".to_string(),
                        description: "Plans with critical risks are blocked".to_string(),
                        condition: "risk_level == 'critical'".to_string(),
                        action: PolicyAction::Deny,
                    },
                    PolicyRule {
                        id: "high-risk-escalate".to_string(),
                        description: "Plans with high risks are escalated".to_string(),
                        condition: "risk_level == 'high'".to_string(),
                        action: PolicyAction::Escalate {
                            to_role: "Risk Committee".to_string(),
                        },
                    },
                ],
                enforcement_level: EnforcementLevel::Mandatory,
            },
        );
    }

    /// Register a routing rule
    pub fn register_routing_rule(&mut self, rule: DecisionRoute) {
        self.routing_rules.insert(rule.id.clone(), rule);
    }

    /// Register a decision policy
    pub fn register_policy(&mut self, policy: DecisionPolicy) {
        self.policies.insert(policy.id.clone(), policy);
    }

    /// Process a decision for an execution plan
    pub async fn process_decision(&self, mut context: DecisionContext) -> DecisionOutcome {
        // Apply routing rules to determine which engines to use
        let routes = self.determine_routes(&context.plan);

        // Apply each routed engine
        for route in routes {
            match route.target_engine {
                DecisionTarget::Validation => {
                    let result = self
                        .validation_engine
                        .validate_execution_plan(&context.plan);
                    context.validation_result = Some(result);
                }
                DecisionTarget::Verification => {
                    let result = self
                        .verification_engine
                        .verify_execution_plan(&context.plan)
                        .await;
                    context.verification_result = Some(result);
                }
                DecisionTarget::RiskAssessment => {
                    let result = self.risk_engine.assess_execution_plan_risks(&context.plan);
                    context.risk_assessment = Some(result);
                }
                _ => {} // Handle other targets as needed
            }
        }

        // Apply policies to make final decision
        self.apply_policies(&context)
    }

    /// Determine which routes apply to a plan
    fn determine_routes(&self, plan: &ExecutionPlanV1) -> Vec<&DecisionRoute> {
        let mut applicable_routes = Vec::new();

        for route in self.routing_rules.values() {
            let matches = match &route.condition {
                RoutingCondition::PlanComplexity {
                    min_tasks,
                    max_tasks,
                } => plan.tasks.len() >= *min_tasks && plan.tasks.len() <= *max_tasks,
                RoutingCondition::RiskLevel { min_level: _ } => {
                    // Would check risk assessment if available
                    false // Simplified for now
                }
                RoutingCondition::ValidationStatus { required_score: _ } => {
                    // Would check validation result if available
                    false // Simplified for now
                }
                RoutingCondition::Custom(condition) => {
                    condition == "always" // Special case for always matching
                }
            };

            if matches {
                applicable_routes.push(route);
            }
        }

        // Sort by priority
        applicable_routes.sort_by_key(|route| match route.priority {
            RoutePriority::Critical => 0,
            RoutePriority::High => 1,
            RoutePriority::Normal => 2,
            RoutePriority::Low => 3,
        });

        applicable_routes
    }

    /// Apply policies to make a decision
    fn apply_policies(&self, context: &DecisionContext) -> DecisionOutcome {
        // Start with allowing the plan
        let mut outcome = DecisionResult::Approved;
        let mut reasoning = "Plan meets all policy requirements".to_string();
        let mut next_steps = Vec::new();
        let mut confidence = 0.9;

        // Check each policy
        for policy in self.policies.values() {
            if matches!(
                policy.enforcement_level,
                EnforcementLevel::Mandatory | EnforcementLevel::Blocking
            ) {
                for rule in &policy.rules {
                    let triggered = self.evaluate_policy_rule(rule, context);

                    if triggered {
                        match &rule.action {
                            PolicyAction::Deny => {
                                outcome = DecisionResult::Rejected;
                                reasoning =
                                    format!("Rejected due to policy rule: {}", rule.description);
                                confidence = 0.95;
                                break;
                            }
                            PolicyAction::Escalate { to_role } => {
                                outcome = DecisionResult::Escalated {
                                    to: to_role.clone(),
                                };
                                reasoning =
                                    format!("Escalated due to policy rule: {}", rule.description);
                                next_steps.push(NextStep {
                                    id: "escalation-step".to_string(),
                                    description: format!("Escalate to {}", to_role),
                                    assignee: to_role.clone(),
                                    due_date: Some(chrono::Utc::now() + chrono::Duration::days(2)),
                                });
                                confidence = 0.8;
                            }
                            PolicyAction::Conditional { conditions } => {
                                outcome = DecisionResult::Conditional {
                                    conditions: conditions.clone(),
                                };
                                reasoning = format!(
                                    "Conditional approval due to policy rule: {}",
                                    rule.description
                                );
                                confidence = 0.7;
                            }
                            _ => {} // Allow and Delegate don't change the default approval
                        }
                    }
                }
            }
        }

        // If we have validation results, adjust confidence
        if let Some(validation) = &context.validation_result {
            confidence *= validation.score;
            if !validation.violations.is_empty() {
                reasoning.push_str(&format!(
                    " with {} validation violations",
                    validation.violations.len()
                ));
            }
        }

        DecisionOutcome {
            decision_id: format!("dec-{}", uuid::Uuid::new_v4()),
            decision_type: DecisionType::PlanApproval,
            outcome,
            reasoning,
            next_steps,
            confidence,
            made_at: chrono::Utc::now(),
            made_by: DecisionMaker::Hybrid {
                automated_confidence_threshold: 0.8,
            },
        }
    }

    /// Evaluate a policy rule against the decision context
    fn evaluate_policy_rule(&self, rule: &PolicyRule, context: &DecisionContext) -> bool {
        // In a real implementation, this would parse and evaluate the condition
        // For now, we'll implement simple checks

        match rule.condition.as_str() {
            "validation_score > 0.8" => {
                if let Some(validation) = &context.validation_result {
                    validation.score > 0.8
                } else {
                    false
                }
            }
            "risk_level == 'critical'" => {
                if let Some(risk_assessment) = &context.risk_assessment {
                    matches!(
                        risk_assessment.risk_level,
                        crate::capabilities::risk_engine::RiskLevel::Critical
                    )
                } else {
                    false
                }
            }
            "risk_level == 'high'" => {
                if let Some(risk_assessment) = &context.risk_assessment {
                    matches!(
                        risk_assessment.risk_level,
                        crate::capabilities::risk_engine::RiskLevel::High
                    )
                } else {
                    false
                }
            }
            _ => false, // Unknown conditions don't trigger
        }
    }

    /// Get routing rules
    pub fn routing_rules(&self) -> &HashMap<String, DecisionRoute> {
        &self.routing_rules
    }

    /// Get policies
    pub fn policies(&self) -> &HashMap<String, DecisionPolicy> {
        &self.policies
    }
}

impl Default for DecisionGateway {
    fn default() -> Self {
        Self::new(
            ValidationEngine::default(),
            VerificationEngine::default(),
            RiskEngine::default(),
        )
    }
}
