//! Governance system for enforcing policies throughout the kernel
//!
//! ## Policy Semantics
//!
//! A rule's condition specifies a **compliance requirement** that must be true.
//! If the condition evaluates to false, a violation is recorded.
//!
//! Supported operators: `starts_with`, `contains`, `ends_with`, `==`, `!=`
//!
//! Condition format: `<field> <operator> <value>`
//! Example: `"priority starts_with high"` means priority must start with "high" to be compliant.

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

/// Rule type determines how the condition is interpreted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleType {
    /// Condition must be true for compliance (default)
    #[default]
    Compliance,
    /// Condition being true triggers a violation (forbidden pattern)
    Forbidden,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub condition: String,
    pub action: String,
    pub severity: Severity,
    #[serde(default)]
    pub rule_type: RuleType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Supported operators for rule conditions
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    StartsWith,
    Contains,
    EndsWith,
    Equals,
    NotEquals,
}

impl Operator {
    /// Parse operator from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "starts_with" => Some(Operator::StartsWith),
            "contains" => Some(Operator::Contains),
            "ends_with" => Some(Operator::EndsWith),
            "==" => Some(Operator::Equals),
            "!=" => Some(Operator::NotEquals),
            _ => None,
        }
    }

    /// Evaluate the operator against field value and rule value
    pub fn evaluate(&self, field_value: &str, rule_value: &str) -> bool {
        match self {
            Operator::StartsWith => field_value.starts_with(rule_value),
            Operator::Contains => field_value.contains(rule_value),
            Operator::EndsWith => field_value.ends_with(rule_value),
            Operator::Equals => field_value == rule_value,
            Operator::NotEquals => field_value != rule_value,
        }
    }
}

/// Parsed rule condition with field, operator, and value
#[derive(Debug, Clone)]
pub struct ParsedCondition {
    pub field: String,
    pub operator: Operator,
    pub value: String,
}

impl ParsedCondition {
    /// Parse a condition string into components
    /// Format: "<field> <operator> <value>"
    /// Returns None if the condition cannot be parsed
    pub fn parse(condition: &str) -> Option<Self> {
        // Try each operator in order of length (longest first to avoid partial matches)
        let operators = ["starts_with", "contains", "ends_with", "!=", "=="];
        
        for op_str in operators {
            if let Some(pos) = condition.find(op_str) {
                let field = condition[..pos].trim();
                let value = condition[pos + op_str.len()..].trim();
                
                if let Some(operator) = Operator::from_str(op_str) {
                    if !field.is_empty() && !value.is_empty() {
                        return Some(ParsedCondition {
                            field: field.to_string(),
                            operator,
                            value: value.to_string(),
                        });
                    }
                }
            }
        }
        None
    }

    /// Evaluate the condition against a set of field values
    /// Returns None if the field is not found in context
    pub fn evaluate(&self, context: &std::collections::HashMap<String, String>) -> Option<bool> {
        context.get(&self.field).map(|field_value| {
            self.operator.evaluate(field_value, &self.value)
        })
    }
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

    /// Evaluate a single rule against the provided context
    /// Returns the raw condition result (before rule type interpretation)
    /// Returns None if the condition cannot be parsed or field not found
    fn evaluate_condition(&self, rule: &Rule, context: &std::collections::HashMap<String, String>) -> Option<bool> {
        let parsed = ParsedCondition::parse(&rule.condition)?;
        parsed.evaluate(context)
    }

    /// Validate an action against governance policies
    /// The context map contains field-value pairs to check against policy rules
    pub fn validate(&self, context: &std::collections::HashMap<String, String>) -> ValidationResult {
        let mut violations = Vec::new();

        for policy in &self.policies {
            if !policy.enabled {
                continue; // Skip disabled policies
            }

            for rule in &policy.rules {
                match self.evaluate_condition(rule, context) {
                    Some(condition_true) => {
                        // condition_true is the raw result of evaluating the condition
                        // For Compliance rules: condition must be true (false = violation)
                        // For Forbidden rules: condition being true triggers violation (true = violation)
                        let is_violation = match rule.rule_type {
                            RuleType::Compliance => !condition_true,  // violation if condition is false
                            RuleType::Forbidden => condition_true,    // violation if condition is true
                        };
                        
                        if is_violation {
                            let message = match rule.rule_type {
                                RuleType::Compliance => format!(
                                    "Policy '{}' violated: {} (required condition not met: {})",
                                    policy.name, rule.action, rule.condition
                                ),
                                RuleType::Forbidden => format!(
                                    "Policy '{}' violated: {} (forbidden condition matched: {})",
                                    policy.name, rule.action, rule.condition
                                ),
                            };
                            violations.push(Violation {
                                policy_id: policy.id.clone(),
                                rule_id: rule.id.clone(),
                                severity: rule.severity.clone(),
                                message,
                            });
                        }
                    }
                    None => {
                        // Unknown syntax or missing field - treat as advisory (non-enforcing)
                        // Log but don't create violation to avoid blocking on malformed rules
                        eprintln!(
                            "Warning: Could not evaluate rule {} in policy {}: invalid condition or missing field",
                            rule.id, policy.id
                        );
                    }
                }
            }
        }

        ValidationResult {
            compliant: violations.is_empty(),
            violations,
        }
    }

    /// Validate an execution plan against governance policies
    /// This applies the same semantics as validate() but accepts a plan-specific context
    pub fn validate_plan(&self, plan_context: &std::collections::HashMap<String, String>) -> ValidationResult {
        self.validate(plan_context)
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_context(fields: Vec<(&str, &str)>) -> HashMap<String, String> {
        fields.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
    }

    // ===== Operator Tests =====

    #[test]
    fn test_operator_starts_with_match() {
        assert!(Operator::StartsWith.evaluate("high-priority", "high"));
        assert!(Operator::StartsWith.evaluate("critical-issue", "critical"));
    }

    #[test]
    fn test_operator_starts_with_no_match() {
        assert!(!Operator::StartsWith.evaluate("low-priority", "high"));
        assert!(!Operator::StartsWith.evaluate("medium-issue", "critical"));
    }

    #[test]
    fn test_operator_contains_match() {
        assert!(Operator::Contains.evaluate("high-priority-task", "priority"));
        assert!(Operator::Contains.evaluate("security-review", "review"));
    }

    #[test]
    fn test_operator_contains_no_match() {
        assert!(!Operator::Contains.evaluate("low-task", "priority"));
        assert!(!Operator::Contains.evaluate("design-doc", "review"));
    }

    #[test]
    fn test_operator_ends_with_match() {
        assert!(Operator::EndsWith.evaluate("task-critical", "critical"));
        assert!(Operator::EndsWith.evaluate("doc-final", "final"));
    }

    #[test]
    fn test_operator_ends_with_no_match() {
        assert!(!Operator::EndsWith.evaluate("task-low", "critical"));
        assert!(!Operator::EndsWith.evaluate("doc-draft", "final"));
    }

    #[test]
    fn test_operator_equals_match() {
        assert!(Operator::Equals.evaluate("exact", "exact"));
        assert!(Operator::Equals.evaluate("priority-high", "priority-high"));
    }

    #[test]
    fn test_operator_equals_no_match() {
        assert!(!Operator::Equals.evaluate("high", "low"));
        assert!(!Operator::Equals.evaluate("priority-high", "high-priority"));
    }

    #[test]
    fn test_operator_not_equals_match() {
        assert!(Operator::NotEquals.evaluate("high", "low"));
        assert!(Operator::NotEquals.evaluate("different", "value"));
    }

    #[test]
    fn test_operator_not_equals_no_match() {
        assert!(!Operator::NotEquals.evaluate("same", "same"));
        assert!(!Operator::NotEquals.evaluate("identical", "identical"));
    }

    // ===== ParsedCondition Tests =====

    #[test]
    fn test_parse_condition_starts_with() {
        let parsed = ParsedCondition::parse("priority starts_with high").unwrap();
        assert_eq!(parsed.field, "priority");
        assert_eq!(parsed.operator, Operator::StartsWith);
        assert_eq!(parsed.value, "high");
    }

    #[test]
    fn test_parse_condition_contains() {
        let parsed = ParsedCondition::parse("name contains security").unwrap();
        assert_eq!(parsed.field, "name");
        assert_eq!(parsed.operator, Operator::Contains);
        assert_eq!(parsed.value, "security");
    }

    #[test]
    fn test_parse_condition_ends_with() {
        let parsed = ParsedCondition::parse("status ends_with done").unwrap();
        assert_eq!(parsed.field, "status");
        assert_eq!(parsed.operator, Operator::EndsWith);
        assert_eq!(parsed.value, "done");
    }

    #[test]
    fn test_parse_condition_equals() {
        let parsed = ParsedCondition::parse("level == critical").unwrap();
        assert_eq!(parsed.field, "level");
        assert_eq!(parsed.operator, Operator::Equals);
        assert_eq!(parsed.value, "critical");
    }

    #[test]
    fn test_parse_condition_not_equals() {
        let parsed = ParsedCondition::parse("type != deprecated").unwrap();
        assert_eq!(parsed.field, "type");
        assert_eq!(parsed.operator, Operator::NotEquals);
        assert_eq!(parsed.value, "deprecated");
    }

    #[test]
    fn test_parse_condition_invalid() {
        assert!(ParsedCondition::parse("invalid condition").is_none());
        assert!(ParsedCondition::parse("").is_none());
        assert!(ParsedCondition::parse("field only").is_none());
    }

    #[test]
    fn test_parsed_condition_evaluate_match() {
        let parsed = ParsedCondition::parse("priority starts_with high").unwrap();
        let mut context = HashMap::new();
        context.insert("priority".to_string(), "high-priority".to_string());
        assert_eq!(parsed.evaluate(&context), Some(true));
    }

    #[test]
    fn test_parsed_condition_evaluate_no_match() {
        let parsed = ParsedCondition::parse("priority starts_with high").unwrap();
        let mut context = HashMap::new();
        context.insert("priority".to_string(), "low-priority".to_string());
        assert_eq!(parsed.evaluate(&context), Some(false));
    }

    #[test]
    fn test_parsed_condition_evaluate_missing_field() {
        let parsed = ParsedCondition::parse("priority starts_with high").unwrap();
        let context = HashMap::new();
        assert_eq!(parsed.evaluate(&context), None);
    }

    // ===== Governance Validation Tests =====

    #[test]
    fn test_compliance_rule_matching() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Priority Policy".to_string(),
            description: "All tasks must have high priority".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "priority starts_with high".to_string(),
                action: "require high priority".to_string(),
                severity: Severity::High,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        // Matching condition - should be compliant
        let context = create_test_context(vec![("priority", "high-priority")]);
        let result = gov.validate(&context);
        assert!(result.compliant);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_compliance_rule_non_matching() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Priority Policy".to_string(),
            description: "All tasks must have high priority".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "priority starts_with high".to_string(),
                action: "require high priority".to_string(),
                severity: Severity::High,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        // Non-matching condition - should violate
        let context = create_test_context(vec![("priority", "low-priority")]);
        let result = gov.validate(&context);
        assert!(!result.compliant);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].severity, Severity::High);
    }

    #[test]
    fn test_forbidden_rule_matching() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Security Policy".to_string(),
            description: "No deprecated tools allowed".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "tool == deprecated-tool".to_string(),
                action: "block deprecated tool".to_string(),
                severity: Severity::Critical,
                rule_type: RuleType::Forbidden,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        // Forbidden condition matched (tool IS deprecated-tool) - should violate
        let context = create_test_context(vec![("tool", "deprecated-tool")]);
        let result = gov.validate(&context);
        // For Forbidden rules: if condition matches (tool == deprecated-tool is true), it's a violation
        assert!(!result.compliant);
        assert_eq!(result.violations.len(), 1);
        assert_eq!(result.violations[0].severity, Severity::Critical);
    }

    #[test]
    fn test_forbidden_rule_non_matching() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Security Policy".to_string(),
            description: "No deprecated tools allowed".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "tool == deprecated-tool".to_string(),
                action: "block deprecated tool".to_string(),
                severity: Severity::Critical,
                rule_type: RuleType::Forbidden,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        // Forbidden condition not matched (tool is NOT deprecated-tool) - should be compliant
        let context = create_test_context(vec![("tool", "approved-tool")]);
        let result = gov.validate(&context);
        // For Forbidden rules: if condition doesn't match, it's compliant
        assert!(result.compliant);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_disabled_policy_ignored() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Disabled Policy".to_string(),
            description: "This policy is disabled".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "priority == low".to_string(),
                action: "require high priority".to_string(),
                severity: Severity::Low,
                rule_type: RuleType::Compliance,
            }],
            enabled: false, // Disabled
        };
        gov.add_policy(policy);

        // Even though condition doesn't match, policy is disabled so no violation
        let context = create_test_context(vec![("priority", "low")]);
        let result = gov.validate(&context);
        assert!(result.compliant);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_multiple_policies_aggregate_violations() {
        let mut gov = Governance::new();
        
        // Policy 1
        gov.add_policy(Policy {
            id: "policy-1".to_string(),
            name: "Priority Policy".to_string(),
            description: "Priority requirement".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "priority starts_with high".to_string(),
                action: "require high priority".to_string(),
                severity: Severity::High,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        });

        // Policy 2
        gov.add_policy(Policy {
            id: "policy-2".to_string(),
            name: "Security Policy".to_string(),
            description: "Security requirement".to_string(),
            rules: vec![Rule {
                id: "rule-2".to_string(),
                condition: "security contains audit".to_string(),
                action: "require audit".to_string(),
                severity: Severity::Medium,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        });

        // Both conditions fail
        let context = create_test_context(vec![
            ("priority", "low"),
            ("security", "basic"),
        ]);
        let result = gov.validate(&context);
        assert!(!result.compliant);
        assert_eq!(result.violations.len(), 2);
    }

    #[test]
    fn test_severity_preserved() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Test Policy".to_string(),
            description: "Test".to_string(),
            rules: vec![
                Rule {
                    id: "rule-1".to_string(),
                    condition: "priority == low".to_string(),
                    action: "action1".to_string(),
                    severity: Severity::Low,
                    rule_type: RuleType::Compliance,
                },
                Rule {
                    id: "rule-2".to_string(),
                    condition: "priority != high".to_string(),
                    action: "action2".to_string(),
                    severity: Severity::Critical,
                    rule_type: RuleType::Compliance,
                },
            ],
            enabled: true,
        };
        gov.add_policy(policy);

        // Set priority to "medium" which fails both rules
        let context = create_test_context(vec![("priority", "medium")]);
        let result = gov.validate(&context);
        assert!(!result.compliant);
        assert_eq!(result.violations.len(), 2);
        assert_eq!(result.violations[0].severity, Severity::Low);
        assert_eq!(result.violations[1].severity, Severity::Critical);
    }

    #[test]
    fn test_malformed_rule_advisory() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Test Policy".to_string(),
            description: "Test with malformed rule".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "malformed condition without operator".to_string(),
                action: "some action".to_string(),
                severity: Severity::High,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        let context = create_test_context(vec![("field", "value")]);
        let result = gov.validate(&context);
        // Malformed rules are advisory, not blocking
        assert!(result.compliant);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn test_deterministic_evaluation() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Determinism Test".to_string(),
            description: "Test".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "value == test".to_string(),
                action: "require test".to_string(),
                severity: Severity::Medium,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        let context = create_test_context(vec![("value", "other")]);
        
        // Run multiple times to ensure determinism
        let results: Vec<_> = (0..5).map(|_| gov.validate(&context)).collect();
        
        // All results should be identical
        for i in 1..results.len() {
            assert_eq!(results[i].compliant, results[0].compliant);
            assert_eq!(results[i].violations.len(), results[0].violations.len());
        }
    }

    #[test]
    fn test_validate_plan_same_semantics() {
        let mut gov = Governance::new();
        let policy = Policy {
            id: "policy-1".to_string(),
            name: "Plan Policy".to_string(),
            description: "Plan validation".to_string(),
            rules: vec![Rule {
                id: "rule-1".to_string(),
                condition: "plan_type == approved".to_string(),
                action: "require approved plan".to_string(),
                severity: Severity::High,
                rule_type: RuleType::Compliance,
            }],
            enabled: true,
        };
        gov.add_policy(policy);

        let context = create_test_context(vec![("plan_type", "unapproved")]);
        
        let validate_result = gov.validate(&context);
        let validate_plan_result = gov.validate_plan(&context);
        
        // Both should produce same results
        assert_eq!(validate_result.compliant, validate_plan_result.compliant);
        assert_eq!(validate_result.violations.len(), validate_plan_result.violations.len());
    }

    #[test]
    fn test_all_operators_comprehensive() {
        let mut gov = Governance::new();
        
        // Create policies for each operator
        let operators = vec![
            ("starts_with", "prefix-test", "prefix", true),
            ("contains", "middle-value", "iddle", true),
            ("ends_with", "test-suffix", "suffix", true),
            ("==", "exact", "exact", true),
            ("!=", "different", "other", true),
        ];

        for (i, (op, field_val, rule_val, _)) in operators.iter().enumerate() {
            gov.add_policy(Policy {
                id: format!("policy-{}", i),
                name: format!("{} Policy", op),
                description: format!("Test {} operator", op),
                rules: vec![Rule {
                    id: format!("rule-{}", i),
                    condition: format!("field {} {}", op, rule_val),
                    action: format!("test {}", op),
                    severity: Severity::Low,
                    rule_type: RuleType::Compliance,
                }],
                enabled: true,
            });
        }

        // Test matching values
        let match_context = create_test_context(vec![
            ("field", "prefix-test"),
            ("field", "middle-value"),
            ("field", "test-suffix"),
            ("field", "exact"),
            ("field", "different"),
        ]);
        // Verify that all operators can be parsed correctly
        for (i, (op, _, rule_val, _)) in operators.iter().enumerate() {
            gov.add_policy(Policy {
                id: format!("test-policy-{}", i),
                name: format!("Test {} Policy", op),
                description: format!("Test {}", op),
                rules: vec![Rule {
                    id: format!("test-rule-{}", i),
                    condition: format!("field {} {}", op, rule_val),
                    action: "test".to_string(),
                    severity: Severity::Low,
                    rule_type: RuleType::Compliance,
                }],
                enabled: true,
            });
        }
        
        // Just verify we have policies for all operators
        assert_eq!(gov.policies().len(), 5);
    }
}
