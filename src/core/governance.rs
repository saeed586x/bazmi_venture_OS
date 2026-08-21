//! Governance system for enforcing policies throughout the kernel

use serde::{Deserialize, Serialize};

/// Governance system for enforcing policies throughout the Venture OS Kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Governance {
    /// Active governance policies
    policies: Vec<Policy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub rules: Vec<Rule>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub condition: String,
    pub action: String,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl Governance {
    /// Create a new governance system
    pub fn new() -> Self {
        Self { policies: vec![] }
    }

    /// Add a new policy
    pub fn add_policy(&mut self, policy: Policy) {
        self.policies.push(policy);
    }

    /// Get all policies
    pub fn policies(&self) -> &[Policy] {
        &self.policies
    }

    /// Validate an action against governance policies
    pub fn validate(&self, action: &str) -> ValidationResult {
        let mut violations = Vec::new();

        // Check action against all enabled policies and their rules
        for policy in &self.policies {
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                let rule_result = self.evaluate_rule(&rule.condition, action);

                if !rule_result.passed {
                    violations.push(Violation {
                        policy_id: policy.id.clone(),
                        rule_id: rule.id.clone(),
                        severity: rule.severity.clone(),
                        message: format!(
                            "Rule '{}' violated: {}. Action: {}",
                            rule.id, rule_result.reason, action
                        ),
                    });
                }
            }
        }

        ValidationResult {
            compliant: violations.is_empty(),
            violations,
        }
    }

    /// Evaluate a single rule condition against an action
    fn evaluate_rule(&self, condition: &str, action: &str) -> RuleEvaluation {
        // Parse and evaluate the condition expression
        // Supports basic conditions like:
        // - "action starts_with 'deploy'"
        // - "action contains 'delete'"
        // - "action == 'specific_action'"
        // - "action ends_with '_prod'"

        let condition = condition.trim();

        // Handle starts_with condition
        if let Some(pattern) = condition.strip_prefix("action starts_with '") {
            if let Some(pattern) = pattern.strip_suffix('\'') {
                return RuleEvaluation {
                    passed: action.starts_with(pattern),
                    reason: if action.starts_with(pattern) {
                        "condition satisfied".to_string()
                    } else {
                        format!("action '{}' does not start with '{}'", action, pattern)
                    },
                };
            }
        }

        // Handle ends_with condition
        if let Some(pattern) = condition.strip_prefix("action ends_with '") {
            if let Some(pattern) = pattern.strip_suffix('\'') {
                return RuleEvaluation {
                    passed: action.ends_with(pattern),
                    reason: if action.ends_with(pattern) {
                        "condition satisfied".to_string()
                    } else {
                        format!("action '{}' does not end with '{}'", action, pattern)
                    },
                };
            }
        }

        // Handle contains condition
        if let Some(pattern) = condition.strip_prefix("action contains '") {
            if let Some(pattern) = pattern.strip_suffix('\'') {
                return RuleEvaluation {
                    passed: action.contains(pattern),
                    reason: if action.contains(pattern) {
                        "condition satisfied".to_string()
                    } else {
                        format!("action '{}' does not contain '{}'", action, pattern)
                    },
                };
            }
        }

        // Handle equality condition
        if let Some(expected) = condition.strip_prefix("action == '") {
            if let Some(expected) = expected.strip_suffix('\'') {
                return RuleEvaluation {
                    passed: action == expected,
                    reason: if action == expected {
                        "condition satisfied".to_string()
                    } else {
                        format!("action '{}' does not equal '{}'", action, expected)
                    },
                };
            }
        }

        // Handle inequality condition
        if let Some(expected) = condition.strip_prefix("action != '") {
            if let Some(expected) = expected.strip_suffix('\'') {
                return RuleEvaluation {
                    passed: action != expected,
                    reason: if action != expected {
                        "condition satisfied".to_string()
                    } else {
                        format!("action '{}' equals forbidden value '{}'", action, expected)
                    },
                };
            }
        }

        // Default: condition syntax not recognized, treat as passed with warning
        RuleEvaluation {
            passed: true,
            reason: format!("unknown condition syntax: {}", condition),
        }
    }

    /// Validate execution plan against all policies
    pub fn validate_plan(&self, plan_description: &str) -> ValidationResult {
        self.validate(plan_description)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuleEvaluation {
    passed: bool,
    reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub compliant: bool,
    pub violations: Vec<Violation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub policy_id: String,
    pub rule_id: String,
    pub severity: Severity,
    pub message: String,
}

impl Default for Governance {
    fn default() -> Self {
        Self::new()
    }
}
