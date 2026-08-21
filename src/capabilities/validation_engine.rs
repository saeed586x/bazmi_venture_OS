//! Validation Engine - validates requirements, plans, and decisions

use crate::capabilities::requirements_engine::Requirement;
use crate::contracts::ExecutionPlanV1;
use crate::core::governance::Governance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation Engine - validates requirements, plans, and decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationEngine {
    /// Reference to governance for policy validation
    governance: Governance,
    /// Custom validation rules
    validation_rules: HashMap<String, ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rule_type: RuleType,
    pub expression: String,
    pub error_message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    Syntax,
    Semantics,
    Governance,
    BusinessLogic,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub violations: Vec<ValidationViolation>,
    pub score: f64, // 0.0 to 1.0
    pub validated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    pub id: String,
    pub rule_id: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub location: String,
    pub suggested_fix: Option<String>,
}

impl ValidationEngine {
    /// Create a new validation engine
    pub fn new(governance: Governance) -> Self {
        Self {
            governance,
            validation_rules: HashMap::new(),
        }
    }

    /// Register a custom validation rule
    pub fn register_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.insert(rule.id.clone(), rule);
    }

    /// Validate an execution plan
    pub fn validate_execution_plan(&self, plan: &ExecutionPlanV1) -> ValidationResult {
        let mut violations = Vec::new();

        // Check required fields
        if plan.id.is_empty() {
            violations.push(ValidationViolation {
                id: "vp-001".to_string(),
                rule_id: "required-field".to_string(),
                description: "Plan ID is required".to_string(),
                severity: ValidationSeverity::Critical,
                location: "id".to_string(),
                suggested_fix: Some("Provide a unique plan ID".to_string()),
            });
        }

        if plan.version.is_empty() {
            violations.push(ValidationViolation {
                id: "vp-002".to_string(),
                rule_id: "required-field".to_string(),
                description: "Plan version is required".to_string(),
                severity: ValidationSeverity::High,
                location: "version".to_string(),
                suggested_fix: Some("Provide a version number".to_string()),
            });
        }

        if plan.intent_reference.is_empty() {
            violations.push(ValidationViolation {
                id: "vp-003".to_string(),
                rule_id: "required-field".to_string(),
                description: "Intent reference is required".to_string(),
                severity: ValidationSeverity::High,
                location: "intent_reference".to_string(),
                suggested_fix: Some("Provide the original intent".to_string()),
            });
        }

        // Validate against governance policies
        let governance_validation = self
            .governance
            .validate(&format!("validate_plan_{}", plan.id));
        if !governance_validation.compliant {
            for violation in governance_validation.violations {
                violations.push(ValidationViolation {
                    id: format!("gov-{}", violation.policy_id),
                    rule_id: violation.policy_id,
                    description: violation.message,
                    severity: match violation.severity {
                        crate::core::governance::Severity::Low => ValidationSeverity::Low,
                        crate::core::governance::Severity::Medium => ValidationSeverity::Medium,
                        crate::core::governance::Severity::High => ValidationSeverity::High,
                        crate::core::governance::Severity::Critical => ValidationSeverity::Critical,
                    },
                    location: "governance".to_string(),
                    suggested_fix: None,
                });
            }
        }

        // Apply custom validation rules
        for rule in self.validation_rules.values() {
            if let Some(violation) = self.apply_custom_rule(rule, plan) {
                violations.push(violation);
            }
        }

        // Calculate validation score
        let score = if violations.is_empty() {
            1.0
        } else {
            let critical_count = violations
                .iter()
                .filter(|v| matches!(v.severity, ValidationSeverity::Critical))
                .count();
            if critical_count > 0 {
                0.0 // Fail completely if critical violations
            } else {
                let high_count = violations
                    .iter()
                    .filter(|v| matches!(v.severity, ValidationSeverity::High))
                    .count();
                let medium_count = violations
                    .iter()
                    .filter(|v| matches!(v.severity, ValidationSeverity::Medium))
                    .count();
                let low_count = violations
                    .iter()
                    .filter(|v| matches!(v.severity, ValidationSeverity::Low))
                    .count();

                let total_weight =
                    critical_count * 10 + high_count * 5 + medium_count * 2 + low_count;
                let max_possible_weight = (violations.len() * 10).max(1); // Avoid division by zero

                1.0 - (total_weight as f64 / max_possible_weight as f64)
            }
        };

        ValidationResult {
            valid: violations
                .iter()
                .all(|v| !matches!(v.severity, ValidationSeverity::Critical)),
            violations,
            score,
            validated_at: chrono::Utc::now(),
        }
    }

    /// Validate a requirement
    pub fn validate_requirement(&self, requirement: &Requirement) -> ValidationResult {
        let mut violations = Vec::new();

        // Check required fields
        if requirement.id.is_empty() {
            violations.push(ValidationViolation {
                id: "vr-001".to_string(),
                rule_id: "required-field".to_string(),
                description: "Requirement ID is required".to_string(),
                severity: ValidationSeverity::Critical,
                location: "id".to_string(),
                suggested_fix: Some("Provide a unique requirement ID".to_string()),
            });
        }

        if requirement.description.is_empty() {
            violations.push(ValidationViolation {
                id: "vr-002".to_string(),
                rule_id: "required-field".to_string(),
                description: "Requirement description is required".to_string(),
                severity: ValidationSeverity::High,
                location: "description".to_string(),
                suggested_fix: Some("Provide a clear description".to_string()),
            });
        }

        // Validate priority and status combinations
        if let (
            crate::capabilities::requirements_engine::RequirementPriority::Critical,
            crate::capabilities::requirements_engine::RequirementStatus::Proposed,
        ) = (&requirement.priority, &requirement.status)
        {
            violations.push(ValidationViolation {
                id: "vr-003".to_string(),
                rule_id: "priority-status-consistency".to_string(),
                description: "Critical priority requirements should not be in proposed status"
                    .to_string(),
                severity: ValidationSeverity::Medium,
                location: "priority-status".to_string(),
                suggested_fix: Some("Move to approved status or reduce priority".to_string()),
            });
        }

        let score = if violations.is_empty() {
            1.0
        } else {
            0.5 // Simplified scoring
        };

        ValidationResult {
            valid: violations
                .iter()
                .all(|v| !matches!(v.severity, ValidationSeverity::Critical)),
            violations,
            score,
            validated_at: chrono::Utc::now(),
        }
    }

    /// Apply a custom validation rule
    fn apply_custom_rule(
        &self,
        rule: &ValidationRule,
        plan: &ExecutionPlanV1,
    ) -> Option<ValidationViolation> {
        // In a real implementation, this would evaluate the rule expression
        // against the plan. For now, we'll implement some basic checks.

        match rule.rule_type {
            RuleType::Syntax => {
                // Check for basic syntax issues
                None // No syntax issues in our structured data
            }
            RuleType::Governance => {
                // Already checked governance above
                None
            }
            RuleType::BusinessLogic => {
                // Example business logic rule: plans with high priority goals should have retry policy
                let has_high_priority_goals = plan.goals.iter().any(|goal| goal.priority > 3);
                let has_retry_policy = plan.retry_policy.is_some();

                if has_high_priority_goals && !has_retry_policy {
                    Some(ValidationViolation {
                        id: format!("bl-{}", rule.id),
                        rule_id: rule.id.clone(),
                        description: rule.error_message.clone(),
                        severity: rule.severity.clone(),
                        location: "retry_policy".to_string(),
                        suggested_fix: Some(
                            "Add a retry policy for high-priority goals".to_string(),
                        ),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get all validation rules
    pub fn rules(&self) -> &HashMap<String, ValidationRule> {
        &self.validation_rules
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new(Governance::new())
    }
}
