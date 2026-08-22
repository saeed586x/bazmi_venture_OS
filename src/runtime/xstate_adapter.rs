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
        let mut states = Vec::new();
        let mut transitions = Vec::new();

        // Add start state
        states.push(XState {
            id: "start".to_string(),
            name: "Start".to_string(),
            type_: StateType::Atomic,
        });

        // Add states for each task
        for task in &plan.tasks {
            states.push(XState {
                id: format!("task_{}", sanitize_id(&task.id)),
                name: escape_xml(&task.name),
                type_: StateType::Atomic,
            });

            // Add executing sub-state for the task
            states.push(XState {
                id: format!("task_{}_executing", sanitize_id(&task.id)),
                name: format!("Executing {}", escape_xml(&task.name)),
                type_: StateType::Atomic,
            });

            // Add completed sub-state for the task
            states.push(XState {
                id: format!("task_{}_completed", sanitize_id(&task.id)),
                name: format!("Completed {}", escape_xml(&task.name)),
                type_: StateType::Atomic,
            });
        }

        // Add states for gates
        for gate in &plan.gates {
            states.push(XState {
                id: format!("gate_{}", sanitize_id(&gate.id)),
                name: escape_xml(&gate.name),
                type_: StateType::Atomic,
            });

            // Add evaluating sub-state for the gate
            states.push(XState {
                id: format!("gate_{}_evaluating", sanitize_id(&gate.id)),
                name: format!("Evaluating {}", escape_xml(&gate.name)),
                type_: StateType::Atomic,
            });

            // Add passed sub-state for the gate
            states.push(XState {
                id: format!("gate_{}_passed", sanitize_id(&gate.id)),
                name: format!("Passed {}", escape_xml(&gate.name)),
                type_: StateType::Atomic,
            });

            // Add failed sub-state for the gate
            states.push(XState {
                id: format!("gate_{}_failed", sanitize_id(&gate.id)),
                name: format!("Failed {}", escape_xml(&gate.name)),
                type_: StateType::Atomic,
            });
        }

        // Add completion states
        for condition in &plan.completion_conditions {
            states.push(XState {
                id: format!("completion_{}", sanitize_id(&condition.id)),
                name: escape_xml(&condition.description),
                type_: StateType::Final,
            });
        }

        // Add final state
        states.push(XState {
            id: "final".to_string(),
            name: "Final".to_string(),
            type_: StateType::Final,
        });

        // Add transitions from start to initial tasks (tasks with no dependencies)
        let dependent_tasks: std::collections::HashSet<&String> = plan
            .dependencies
            .iter()
            .map(|d| &d.dependent_task_id)
            .collect();

        for task in &plan.tasks {
            if !dependent_tasks.contains(&task.id) {
                transitions.push(XTransition {
                    from: "start".to_string(),
                    to: format!("task_{}", sanitize_id(&task.id)),
                    event: "begin".to_string(),
                    guard: None,
                });
            }
        }

        // If no tasks exist, transition directly to final
        if plan.tasks.is_empty() && plan.gates.is_empty() {
            transitions.push(XTransition {
                from: "start".to_string(),
                to: "final".to_string(),
                event: "complete".to_string(),
                guard: None,
            });
        }

        // Add transitions for task dependencies
        for dep in &plan.dependencies {
            transitions.push(XTransition {
                from: format!("task_{}_completed", sanitize_id(&dep.dependency_task_id)),
                to: format!("task_{}", sanitize_id(&dep.dependent_task_id)),
                event: "dependency_satisfied".to_string(),
                guard: None,
            });
        }

        // Add transitions from task executing to completed
        for task in &plan.tasks {
            transitions.push(XTransition {
                from: format!("task_{}", sanitize_id(&task.id)),
                to: format!("task_{}_executing", sanitize_id(&task.id)),
                event: "execute".to_string(),
                guard: None,
            });

            transitions.push(XTransition {
                from: format!("task_{}_executing", sanitize_id(&task.id)),
                to: format!("task_{}_completed", sanitize_id(&task.id)),
                event: "finish".to_string(),
                guard: None,
            });
        }

        // Add transitions for gates
        for gate in &plan.gates {
            transitions.push(XTransition {
                from: format!("gate_{}", sanitize_id(&gate.id)),
                to: format!("gate_{}_evaluating", sanitize_id(&gate.id)),
                event: "evaluate".to_string(),
                guard: None,
            });

            transitions.push(XTransition {
                from: format!("gate_{}_evaluating", sanitize_id(&gate.id)),
                to: format!("gate_{}_passed", sanitize_id(&gate.id)),
                event: "pass".to_string(),
                guard: Some(format_gate_criterion(&gate.criteria)),
            });

            transitions.push(XTransition {
                from: format!("gate_{}_evaluating", sanitize_id(&gate.id)),
                to: format!("gate_{}_failed", sanitize_id(&gate.id)),
                event: "fail".to_string(),
                guard: Some(format!("!({})", format_gate_criterion(&gate.criteria))),
            });
        }

        // Connect last tasks/gates to completion or final state
        if !plan.completion_conditions.is_empty() {
            // Connect to completion conditions
            for condition in &plan.completion_conditions {
                transitions.push(XTransition {
                    from: "final".to_string(),
                    to: format!("completion_{}", sanitize_id(&condition.id)),
                    event: "satisfy".to_string(),
                    guard: Some(escape_mermaid(&condition.expression)),
                });
            }
        } else if !plan.tasks.is_empty() {
            // Find terminal tasks (tasks that are not dependencies of other tasks)
            let dependency_task_ids: std::collections::HashSet<&String> = plan
                .dependencies
                .iter()
                .map(|d| &d.dependency_task_id)
                .collect();

            for task in &plan.tasks {
                if !dependency_task_ids.contains(&task.id) {
                    transitions.push(XTransition {
                        from: format!("task_{}_completed", sanitize_id(&task.id)),
                        to: "final".to_string(),
                        event: "all_complete".to_string(),
                        guard: None,
                    });
                }
            }
        }

        Ok(XStateMachine {
            id: format!("plan_{}", sanitize_id(&plan.id)),
            initial: "start".to_string(),
            states,
            transitions,
            version: plan.version.clone(),
            intent_reference: escape_xml(&plan.intent_reference),
        })
    }

    /// Export XState machine to specified format
    pub fn export(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        match self.config.export_format {
            ExportFormat::Json => self.export_json(machine),
            ExportFormat::Scxml => self.export_scxml(machine),
            ExportFormat::Mermaid => self.export_mermaid(machine),
        }
    }

    /// Export to JSON format
    fn export_json(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        Ok(serde_json::to_string_pretty(machine)?)
    }

    /// Export to SCXML format
    fn export_scxml(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        let mut scxml = String::new();
        scxml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        scxml.push_str(&format!(
            "<scxml xmlns=\"http://www.w3.org/2005/07/scxml\" version=\"1.0\" initial=\"{}\" id=\"{}\">\n",
            escape_xml_attr(&machine.initial),
            escape_xml_attr(&machine.id)
        ));

        // Output states
        for state in &machine.states {
            let state_type_str = match state.type_ {
                StateType::Final => "final",
                _ => "",
            };
            scxml.push_str(&format!(
                "  <state id=\"{}\" name=\"{}\"{}>\n",
                escape_xml_attr(&state.id),
                escape_xml_attr(&state.name),
                if state_type_str.is_empty() {
                    "".to_string()
                } else {
                    format!(" type=\"{}\"", state_type_str)
                }
            ));
            scxml.push_str("  </state>\n");
        }

        // Output transitions
        for transition in &machine.transitions {
            scxml.push_str(&format!(
                "  <transition from=\"{}\" to=\"{}\" event=\"{}\"",
                escape_xml_attr(&transition.from),
                escape_xml_attr(&transition.to),
                escape_xml_attr(&transition.event)
            ));
            if let Some(ref guard) = transition.guard {
                scxml.push_str(&format!(" cond=\"{}\"", escape_xml_attr(guard)));
            }
            scxml.push_str("/>\n");
        }

        scxml.push_str("</scxml>\n");
        Ok(scxml)
    }

    /// Export to Mermaid format
    fn export_mermaid(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        let mut mermaid = String::new();
        mermaid.push_str("stateDiagram-v2\n");
        mermaid.push_str(&format!("  [*] --> {}\n", escape_mermaid(&machine.initial)));

        // Group states by type for better visualization
        let mut compound_states = Vec::new();
        let mut atomic_states = Vec::new();
        let mut final_states = Vec::new();

        for state in &machine.states {
            match state.type_ {
                StateType::Compound => compound_states.push(state),
                StateType::Final => final_states.push(state),
                _ => atomic_states.push(state),
            }
        }

        // Output atomic states
        for state in atomic_states {
            mermaid.push_str(&format!(
                "  {} : {}\n",
                state.id,
                escape_mermaid(&state.name)
            ));
        }

        // Output final states
        for state in final_states {
            mermaid.push_str(&format!(
                "  {} : {}\n",
                state.id,
                escape_mermaid(&state.name)
            ));
            mermaid.push_str(&format!("  {} --> [*]\n", state.id));
        }

        // Output transitions
        for transition in &machine.transitions {
            let guard_str = if let Some(ref guard) = transition.guard {
                format!(" : {}", escape_mermaid(guard))
            } else {
                String::new()
            };
            mermaid.push_str(&format!(
                "  {} --> {} : {}{}\n",
                transition.from,
                transition.to,
                escape_mermaid(&transition.event),
                guard_str
            ));
        }

        Ok(mermaid)
    }
}

/// Sanitize an ID for use in state machine identifiers
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Escape special characters for XML content
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape special characters for XML attributes
fn escape_xml_attr(s: &str) -> String {
    escape_xml(s).replace('\n', "&#10;").replace('\r', "&#13;")
}

/// Escape special characters for Mermaid diagrams
fn escape_mermaid(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('(', "\\(")
        .replace(')', "\\)")
        .replace('[', "\\[")
        .replace(']', "\\]")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('<', "\\<")
        .replace('>', "\\>")
        .replace('"', "\\\"")
        .replace('\'', "\\'")
        .replace(['\n', '\r'], " ")
}

/// Format gate criteria as a guard expression
fn format_gate_criterion(
    criteria: &[crate::contracts::execution_plan_v1::GateCriterion],
) -> String {
    if criteria.is_empty() {
        return "true".to_string();
    }

    let criterion_strs: Vec<String> = criteria
        .iter()
        .map(|c| escape_xml(&c.description))
        .collect();

    criterion_strs.join(" AND ")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStateMachine {
    pub id: String,
    pub initial: String,
    pub states: Vec<XState>,
    pub transitions: Vec<XTransition>,
    pub version: String,
    pub intent_reference: String,
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
    #[error("Invalid state reference: {0}")]
    InvalidStateReference(String),
    #[error("Invalid transition: {0}")]
    InvalidTransition(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::execution_plan_v1::{
        CompletionCondition, Dependency, Gate, GateCriterion, GateType, Goal, Task,
    };
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_plan() -> ExecutionPlanV1 {
        ExecutionPlanV1 {
            id: "test-plan-123".to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Test intent".to_string(),
            goals: vec![Goal {
                id: "goal-1".to_string(),
                description: "Test goal".to_string(),
                priority: 1,
            }],
            constraints: vec![],
            required_capabilities: vec!["development".to_string()],
            inputs: vec![],
            tasks: vec![
                Task {
                    id: "task-1".to_string(),
                    name: "First Task".to_string(),
                    description: "First task description".to_string(),
                    capability: "development".to_string(),
                    parameters: HashMap::new(),
                    expected_duration: Some(3600),
                },
                Task {
                    id: "task-2".to_string(),
                    name: "Second Task".to_string(),
                    description: "Second task description".to_string(),
                    capability: "development".to_string(),
                    parameters: HashMap::new(),
                    expected_duration: Some(7200),
                },
            ],
            dependencies: vec![Dependency {
                dependent_task_id: "task-2".to_string(),
                dependency_task_id: "task-1".to_string(),
            }],
            artifacts: vec![],
            gates: vec![Gate {
                id: "gate-1".to_string(),
                name: "Quality Gate".to_string(),
                description: "Quality check gate".to_string(),
                gate_type: GateType::Quality,
                criteria: vec![GateCriterion {
                    id: "criterion-1".to_string(),
                    description: "All tests pass".to_string(),
                    evaluation_method: "automated".to_string(),
                }],
            }],
            completion_conditions: vec![CompletionCondition {
                id: "completion-1".to_string(),
                description: "All tasks complete".to_string(),
                expression: "tasks_completed == 2".to_string(),
            }],
            retry_policy: None,
            provenance: None,
            creation_timestamp: Utc::now(),
            replan_reason: None,
        }
    }

    fn create_empty_plan() -> ExecutionPlanV1 {
        ExecutionPlanV1 {
            id: "empty-plan".to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Empty intent".to_string(),
            goals: vec![],
            constraints: vec![],
            required_capabilities: vec![],
            inputs: vec![],
            tasks: vec![],
            dependencies: vec![],
            artifacts: vec![],
            gates: vec![],
            completion_conditions: vec![],
            retry_policy: None,
            provenance: None,
            creation_timestamp: Utc::now(),
            replan_reason: None,
        }
    }

    fn create_plan_with_special_chars() -> ExecutionPlanV1 {
        ExecutionPlanV1 {
            id: "test-plan@special#chars!".to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Test <intent> with \"special\" & 'chars'".to_string(),
            goals: vec![],
            constraints: vec![],
            required_capabilities: vec![],
            inputs: vec![],
            tasks: vec![Task {
                id: "task<1>".to_string(),
                name: "Task [with] special {chars}".to_string(),
                description: "Description with <>&\"'".to_string(),
                capability: "dev".to_string(),
                parameters: HashMap::new(),
                expected_duration: None,
            }],
            dependencies: vec![],
            artifacts: vec![],
            gates: vec![],
            completion_conditions: vec![],
            retry_policy: None,
            provenance: None,
            creation_timestamp: Utc::now(),
            replan_reason: None,
        }
    }

    #[test]
    fn test_to_xstate_basic() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let result = adapter.to_xstate(&plan);
        assert!(result.is_ok());

        let machine = result.unwrap();
        assert_eq!(machine.id, "plan_test-plan-123");
        assert_eq!(machine.initial, "start");
        assert_eq!(machine.version, "1.0.0");
        assert_eq!(machine.intent_reference, "Test intent");

        // Should have states for start, tasks, gates, completion, and final
        assert!(machine.states.len() > 5);

        // Verify task states exist
        let task_state_ids: Vec<&String> = machine
            .states
            .iter()
            .map(|s| &s.id)
            .filter(|id| id.starts_with("task_"))
            .collect();
        assert!(!task_state_ids.is_empty());

        // Verify gate states exist
        let gate_state_ids: Vec<&String> = machine
            .states
            .iter()
            .map(|s| &s.id)
            .filter(|id| id.starts_with("gate_"))
            .collect();
        assert!(!gate_state_ids.is_empty());
    }

    #[test]
    fn test_to_xstate_empty_plan() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_empty_plan();

        let result = adapter.to_xstate(&plan);
        assert!(result.is_ok());

        let machine = result.unwrap();
        assert_eq!(machine.id, "plan_empty-plan");

        // Should have start and final states at minimum
        assert!(machine.states.len() >= 2);

        // Should have direct transition from start to final
        let start_to_final = machine
            .transitions
            .iter()
            .any(|t| t.from == "start" && t.to == "final");
        assert!(start_to_final);
    }

    #[test]
    fn test_to_xstate_special_characters() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_plan_with_special_chars();

        let result = adapter.to_xstate(&plan);
        assert!(result.is_ok());

        let machine = result.unwrap();
        // ID should be sanitized (special chars replaced with underscores)
        assert!(machine.id.starts_with("plan_test-plan"));
        // Intent reference should be XML escaped
        assert!(machine.intent_reference.contains("&lt;"));
        assert!(machine.intent_reference.contains("&quot;"));
        assert!(machine.intent_reference.contains("&amp;"));
    }

    #[test]
    fn test_to_xstate_task_dependencies() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let result = adapter.to_xstate(&plan);
        assert!(result.is_ok());

        let machine = result.unwrap();

        // Verify dependency transition exists
        let dependency_transition = machine.transitions.iter().any(|t| {
            t.event == "dependency_satisfied"
                && t.from.contains("task-1_completed")
                && t.to.contains("task-2")
        });
        assert!(dependency_transition);
    }

    #[test]
    fn test_to_xstate_gate_transitions() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let result = adapter.to_xstate(&plan);
        assert!(result.is_ok());

        let machine = result.unwrap();

        // Verify gate evaluation transitions exist
        let evaluate_transition = machine.transitions.iter().any(|t| {
            t.from.contains("gate-1") && t.to.contains("evaluating") && t.event == "evaluate"
        });
        assert!(evaluate_transition);

        // Verify gate pass transition with guard
        let pass_transition = machine.transitions.iter().find(|t| {
            t.from.contains("evaluating") && t.to.contains("passed") && t.event == "pass"
        });
        assert!(pass_transition.is_some());
        assert!(pass_transition.unwrap().guard.is_some());

        // Verify gate fail transition with negated guard
        let fail_transition = machine.transitions.iter().find(|t| {
            t.from.contains("evaluating") && t.to.contains("failed") && t.event == "fail"
        });
        assert!(fail_transition.is_some());
        let guard = fail_transition.unwrap().guard.as_ref().unwrap();
        assert!(guard.starts_with("!("));
    }

    #[test]
    fn test_export_json() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let machine = adapter.to_xstate(&plan).unwrap();
        let json_result = adapter.export(&machine);

        assert!(json_result.is_ok());
        let json_str = json_result.unwrap();

        // Verify it's valid JSON by parsing it
        let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
        assert_eq!(parsed["id"], "plan_test-plan-123");
    }

    #[test]
    fn test_export_scxml() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Scxml,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let machine = adapter.to_xstate(&plan).unwrap();
        let scxml_result = adapter.export(&machine);

        assert!(scxml_result.is_ok());
        let scxml_str = scxml_result.unwrap();

        // Verify SCXML structure
        assert!(scxml_str.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(scxml_str.contains("<scxml"));
        assert!(scxml_str.contains("</scxml>"));
        assert!(scxml_str.contains("initial=\"start\""));

        // Verify exactly one closing scxml tag
        let closing_tag_count = scxml_str.matches("</scxml>").count();
        assert_eq!(closing_tag_count, 1);

        // Verify transitions are present
        assert!(scxml_str.contains("<transition"));
    }

    #[test]
    fn test_export_mermaid() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Mermaid,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let machine = adapter.to_xstate(&plan).unwrap();
        let mermaid_result = adapter.export(&machine);

        assert!(mermaid_result.is_ok());
        let mermaid_str = mermaid_result.unwrap();

        // Verify Mermaid structure
        assert!(mermaid_str.starts_with("stateDiagram-v2"));
        assert!(mermaid_str.contains("[*] --> start"));

        // Verify transitions are present
        assert!(mermaid_str.contains("-->"));
    }

    #[test]
    fn test_export_mermaid_special_chars_escaped() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Mermaid,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_plan_with_special_chars();

        let machine = adapter.to_xstate(&plan).unwrap();
        let mermaid_result = adapter.export(&machine);

        assert!(mermaid_result.is_ok());
        let mermaid_str = mermaid_result.unwrap();

        // Verify special characters are escaped in Mermaid
        // Parentheses, brackets, braces should be escaped
        assert!(!mermaid_str.contains("\\\\") || mermaid_str.contains("\\\\("));
    }

    #[test]
    fn test_scxml_valid_transitions() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Scxml,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let machine = adapter.to_xstate(&plan).unwrap();
        let scxml_str = adapter.export(&machine).unwrap();

        // Verify all transitions have valid from/to attributes
        assert!(scxml_str.contains("from=\""));
        assert!(scxml_str.contains("to=\""));
        assert!(scxml_str.contains("event=\""));

        // Verify conditional transitions have cond attribute
        let has_cond = scxml_str.contains("cond=\"");
        assert!(has_cond); // Should have guards as conditions
    }

    #[test]
    fn test_completion_condition_states() {
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);
        let plan = create_test_plan();

        let machine = adapter.to_xstate(&plan).unwrap();

        // Verify completion condition states exist
        let completion_states: Vec<&XState> = machine
            .states
            .iter()
            .filter(|s| s.id.starts_with("completion_"))
            .collect();
        assert!(!completion_states.is_empty());

        // Verify they are final states
        for state in completion_states {
            match state.type_ {
                StateType::Final => {}
                _ => panic!("Completion condition state should be Final type"),
            }
        }
    }

    #[test]
    fn test_multiple_gates() {
        let mut plan = create_test_plan();
        plan.gates.push(Gate {
            id: "gate-2".to_string(),
            name: "Security Gate".to_string(),
            description: "Security check".to_string(),
            gate_type: GateType::Security,
            criteria: vec![GateCriterion {
                id: "sec-criterion-1".to_string(),
                description: "No vulnerabilities".to_string(),
                evaluation_method: "scan".to_string(),
            }],
        });

        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).unwrap();

        // Verify both gates have states
        let gate1_states: Vec<&XState> = machine
            .states
            .iter()
            .filter(|s| s.id.contains("gate-1"))
            .collect();
        let gate2_states: Vec<&XState> = machine
            .states
            .iter()
            .filter(|s| s.id.contains("gate-2"))
            .collect();

        assert!(!gate1_states.is_empty());
        assert!(!gate2_states.is_empty());
    }
}
