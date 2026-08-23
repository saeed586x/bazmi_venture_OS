//! XState adapter for state machine visualization

use crate::contracts::execution_plan_v1::{Gate, Task};
use crate::contracts::ExecutionPlanV1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
        if plan.tasks.is_empty() {
            // Return a minimal valid state machine for empty plans
            return Ok(XStateMachine {
                id: escape_xml_attribute(&format!("plan_{}", plan.id)),
                initial: "idle".to_string(),
                states: vec![XState {
                    id: "idle".to_string(),
                    name: "Idle".to_string(),
                    type_: StateType::Final,
                }],
                transitions: vec![],
            });
        }

        let mut states: Vec<XState> = Vec::new();
        let mut transitions: Vec<XTransition> = Vec::new();

        // Build a map of task dependencies
        let mut dependency_map: HashMap<String, Vec<String>> = HashMap::new();
        for dep in &plan.dependencies {
            dependency_map
                .entry(dep.dependent_task_id.clone())
                .or_default()
                .push(dep.dependency_task_id.clone());
        }

        // Add start state
        states.push(XState {
            id: "start".to_string(),
            name: "Start".to_string(),
            type_: StateType::Atomic,
        });

        // Find entry tasks (tasks with no dependencies)
        let mut dependent_tasks: HashMap<String, bool> = HashMap::new();
        for dep in &plan.dependencies {
            dependent_tasks.insert(dep.dependent_task_id.clone(), true);
        }

        let entry_tasks: Vec<&Task> = plan
            .tasks
            .iter()
            .filter(|t| !dependent_tasks.contains_key(&t.id))
            .collect();

        // Add transitions from start to entry tasks
        for task in &entry_tasks {
            transitions.push(XTransition {
                from: "start".to_string(),
                to: escape_xml_attribute(&format!("task_{}", task.id)),
                event: "start_task".to_string(),
                guard: None,
            });
        }

        // Add states and transitions for each task
        for task in &plan.tasks {
            let task_id = escape_xml_attribute(&format!("task_{}", task.id));
            let task_name = escape_xml_attribute(&task.name);

            states.push(XState {
                id: task_id.clone(),
                name: task_name,
                type_: StateType::Atomic,
            });

            // Add transition to gate state if task has gates
            let task_gates: Vec<&Gate> = plan
                .gates
                .iter()
                .filter(|g| g.name.contains(&task.name))
                .collect();

            if !task_gates.is_empty() {
                let gate_id = escape_xml_attribute(&format!("gate_{}", task_gates[0].id));
                transitions.push(XTransition {
                    from: task_id.clone(),
                    to: gate_id.clone(),
                    event: "task_complete".to_string(),
                    guard: Some(format!("task_{}_success", escape_xml_attribute(&task.id))),
                });

                states.push(XState {
                    id: gate_id.clone(),
                    name: escape_xml_attribute(&task_gates[0].name),
                    type_: StateType::Atomic,
                });

                // Transition from gate to next tasks
                let next_tasks: Vec<&Task> = plan
                    .tasks
                    .iter()
                    .filter(|t| {
                        dependency_map
                            .get(&t.id)
                            .is_some_and(|deps| deps.contains(&task.id))
                    })
                    .collect();

                for next_task in &next_tasks {
                    transitions.push(XTransition {
                        from: gate_id.clone(),
                        to: escape_xml_attribute(&format!("task_{}", next_task.id)),
                        event: "gate_passed".to_string(),
                        guard: Some(format!(
                            "gate_{}_passed",
                            escape_xml_attribute(&task_gates[0].id)
                        )),
                    });
                }
            } else {
                // No gate, transition directly to next tasks
                let next_tasks: Vec<&Task> = plan
                    .tasks
                    .iter()
                    .filter(|t| {
                        dependency_map
                            .get(&t.id)
                            .is_some_and(|deps| deps.contains(&task.id))
                    })
                    .collect();

                for next_task in &next_tasks {
                    transitions.push(XTransition {
                        from: task_id.clone(),
                        to: escape_xml_attribute(&format!("task_{}", next_task.id)),
                        event: "task_complete".to_string(),
                        guard: Some(format!("task_{}_success", escape_xml_attribute(&task.id))),
                    });
                }
            }
        }

        // Add completion state
        let has_exit_tasks = plan.tasks.iter().any(|t| {
            !plan.tasks.iter().any(|other| {
                dependency_map
                    .get(&other.id)
                    .is_some_and(|deps| deps.contains(&t.id))
            })
        });

        if has_exit_tasks {
            states.push(XState {
                id: "complete".to_string(),
                name: "Complete".to_string(),
                type_: StateType::Final,
            });

            // Find exit tasks (tasks that no other task depends on)
            let exit_tasks: Vec<&Task> = plan
                .tasks
                .iter()
                .filter(|t| {
                    !plan.tasks.iter().any(|other| {
                        dependency_map
                            .get(&other.id)
                            .is_some_and(|deps| deps.contains(&t.id))
                    })
                })
                .collect();

            for task in &exit_tasks {
                transitions.push(XTransition {
                    from: escape_xml_attribute(&format!("task_{}", task.id)),
                    to: "complete".to_string(),
                    event: "all_done".to_string(),
                    guard: None,
                });
            }
        }

        Ok(XStateMachine {
            id: escape_xml_attribute(&format!("plan_{}", plan.id)),
            initial: "start".to_string(),
            states,
            transitions,
        })
    }

    /// Export XState machine to specified format
    pub fn export(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        match self.config.export_format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(machine)?),
            ExportFormat::Scxml => self.export_scxml(machine),
            ExportFormat::Mermaid => self.export_mermaid(machine),
        }
    }

    /// Export to SCXML format with proper escaping
    fn export_scxml(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str(&format!(
            "<scxml xmlns=\"http://www.w3.org/2005/07/scxml\" version=\"1.0\" initial=\"{}\">\n",
            escape_xml_attribute(&machine.initial)
        ));

        // Add states
        for state in &machine.states {
            let state_type = match state.type_ {
                StateType::Final => " final=\"true\"",
                _ => "",
            };
            xml.push_str(&format!(
                "  <state id=\"{}\" name=\"{}{}\">\n",
                escape_xml_attribute(&state.id),
                escape_xml_attribute(&state.name),
                state_type
            ));
            xml.push_str("  </state>\n");
        }

        // Add transitions
        for transition in &machine.transitions {
            let guard_attr = transition
                .guard
                .as_ref()
                .map(|g| format!(" cond=\"{}\"", escape_xml_attribute(g)))
                .unwrap_or_default();

            xml.push_str(&format!(
                "  <transition from=\"{}\" to=\"{}\" event=\"{}\"{} />\n",
                escape_xml_attribute(&transition.from),
                escape_xml_attribute(&transition.to),
                escape_xml_attribute(&transition.event),
                guard_attr
            ));
        }

        xml.push_str("</scxml>");
        Ok(xml)
    }

    /// Export to Mermaid state diagram format with proper escaping
    fn export_mermaid(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        let mut mermaid = String::new();
        mermaid.push_str("stateDiagram-v2\n");
        mermaid.push_str(&format!("  [*] --> {}\n", escape_mermaid(&machine.initial)));

        // Add states
        for state in &machine.states {
            let state_marker = match state.type_ {
                StateType::Final => " {*}",
                _ => "",
            };
            mermaid.push_str(&format!(
                "  {} : {}{}\n",
                escape_mermaid(&state.id),
                escape_mermaid(&state.name),
                state_marker
            ));
        }

        // Add transitions
        for transition in &machine.transitions {
            let guard = transition
                .guard
                .as_ref()
                .map(|g| format!(" : {}", escape_mermaid(g)))
                .unwrap_or_default();

            mermaid.push_str(&format!(
                "  {} --> {}{}\n",
                escape_mermaid(&transition.from),
                escape_mermaid(&transition.to),
                guard
            ));
        }

        Ok(mermaid)
    }
}

/// Escape XML special characters in attribute values
fn escape_xml_attribute(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape Mermaid-sensitive characters
fn escape_mermaid(s: &str) -> String {
    // Replace problematic characters for Mermaid diagrams
    s.replace(['(', ')', '[', ']', '{', '}', '#', ':', ';'], "_")
        .replace('\n', " ")
        .replace('\r', "")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XStateMachine {
    pub id: String,
    pub initial: String,
    pub states: Vec<XState>,
    pub transitions: Vec<XTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XState {
    pub id: String,
    pub name: String,
    pub type_: StateType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[error("Invalid plan: {0}")]
    InvalidPlan(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::execution_plan_v1::{Dependency, GateType};
    use chrono::Utc;
    use std::collections::HashMap;

    fn create_test_plan() -> ExecutionPlanV1 {
        let mut params = HashMap::new();
        params.insert("key".to_string(), serde_json::json!("value"));

        ExecutionPlanV1 {
            id: "test-plan-123".to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Test intent".to_string(),
            goals: vec![],
            constraints: vec![],
            required_capabilities: vec!["capability1".to_string()],
            inputs: vec![],
            tasks: vec![
                Task {
                    id: "task1".to_string(),
                    name: "Task One".to_string(),
                    description: "First task".to_string(),
                    capability: "capability1".to_string(),
                    parameters: params.clone(),
                    expected_duration: Some(60),
                },
                Task {
                    id: "task2".to_string(),
                    name: "Task Two".to_string(),
                    description: "Second task".to_string(),
                    capability: "capability1".to_string(),
                    parameters: params.clone(),
                    expected_duration: Some(60),
                },
            ],
            dependencies: vec![Dependency {
                dependent_task_id: "task2".to_string(),
                dependency_task_id: "task1".to_string(),
            }],
            artifacts: vec![],
            gates: vec![Gate {
                id: "gate1".to_string(),
                name: "Task One Validation".to_string(),
                description: "Validate task one output".to_string(),
                gate_type: GateType::Quality,
                criteria: vec![],
            }],
            completion_conditions: vec![],
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

    #[test]
    fn test_to_xstate_non_empty_plan() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter
            .to_xstate(&plan)
            .expect("Should convert plan to XState");

        assert_eq!(machine.id, "plan_test-plan-123");
        assert_eq!(machine.initial, "start");
        assert!(!machine.states.is_empty());
        assert!(!machine.transitions.is_empty());

        // Verify start state exists
        let start_state = machine.states.iter().find(|s| s.id == "start");
        assert!(start_state.is_some());

        // Verify task states exist
        let task1_state = machine.states.iter().find(|s| s.id == "task_task1");
        assert!(task1_state.is_some());

        let task2_state = machine.states.iter().find(|s| s.id == "task_task2");
        assert!(task2_state.is_some());
    }

    #[test]
    fn test_to_xstate_empty_plan() {
        let plan = create_empty_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should handle empty plan");

        assert_eq!(machine.initial, "idle");
        assert_eq!(machine.states.len(), 1);
        assert_eq!(machine.states[0].type_, StateType::Final);
        assert!(machine.transitions.is_empty());
    }

    #[test]
    fn test_export_json() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");
        let json = adapter.export(&machine).expect("Should export to JSON");

        // Verify JSON is parseable
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
        assert!(parsed.get("id").is_some());
        assert!(parsed.get("states").is_some());
        assert!(parsed.get("transitions").is_some());
    }

    #[test]
    fn test_export_scxml() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Scxml,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");
        let scxml = adapter.export(&machine).expect("Should export to SCXML");

        // Verify SCXML structure
        assert!(scxml.contains("<?xml version=\"1.0\""));
        assert!(scxml.contains("<scxml"));
        assert!(scxml.contains("</scxml>"));
        assert!(scxml.contains("initial=\"start\""));

        // Count closing tags - should have exactly one </scxml>
        let close_count = scxml.matches("</scxml>").count();
        assert_eq!(close_count, 1, "Should have exactly one closing scxml tag");
    }

    #[test]
    fn test_export_mermaid() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Mermaid,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");
        let mermaid = adapter.export(&machine).expect("Should export to Mermaid");

        // Verify Mermaid structure
        assert!(mermaid.starts_with("stateDiagram-v2"));
        assert!(mermaid.contains("[*]"));
        assert!(mermaid.contains("-->"));
    }

    #[test]
    fn test_special_characters_in_ids() {
        let mut params = HashMap::new();
        params.insert("key".to_string(), serde_json::json!("value"));

        let plan = ExecutionPlanV1 {
            id: "plan-with-special&chars<test>".to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Test".to_string(),
            goals: vec![],
            constraints: vec![],
            required_capabilities: vec![],
            inputs: vec![],
            tasks: vec![Task {
                id: "task[1]".to_string(),
                name: "Task#1: Test".to_string(),
                description: "Test task".to_string(),
                capability: "cap".to_string(),
                parameters: params,
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
        };

        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter
            .to_xstate(&plan)
            .expect("Should handle special chars");

        // Verify ID is escaped
        assert!(machine.id.contains("plan-with-special"));

        // Export to SCXML and verify escaping
        let scxml_config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Scxml,
        };
        let scxml_adapter = XStateAdapter::new(scxml_config);
        let scxml = scxml_adapter.export(&machine).expect("Should export SCXML");

        // Verify XML entities are present
        assert!(scxml.contains("&amp;") || !scxml.contains("&"));
    }

    #[test]
    fn test_dependency_transitions() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");

        // Verify there's a transition from task1 to task2 (via gate or direct)
        let has_task_transition = machine.transitions.iter().any(|t| {
            t.from.contains("task_task1") && t.to.contains("task_task2")
                || t.from.contains("task_task1") && t.to.contains("gate_")
        });
        assert!(
            has_task_transition,
            "Should have transition representing task dependency"
        );
    }

    #[test]
    fn test_gate_states() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");

        // Verify gate state exists
        let has_gate_state = machine.states.iter().any(|s| s.id.contains("gate_"));
        assert!(has_gate_state, "Should have gate state");
    }

    #[test]
    fn test_json_parseability() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");
        let json = adapter.export(&machine).expect("Should export to JSON");

        // Parse and verify all required fields
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert!(parsed["id"].is_string());
        assert!(parsed["initial"].is_string());
        assert!(parsed["states"].is_array());
        assert!(parsed["transitions"].is_array());

        let states = parsed["states"].as_array().unwrap();
        for state in states {
            assert!(state["id"].is_string());
            assert!(state["name"].is_string());
        }

        let transitions = parsed["transitions"].as_array().unwrap();
        for transition in transitions {
            assert!(transition["from"].is_string());
            assert!(transition["to"].is_string());
            assert!(transition["event"].is_string());
        }
    }

    #[test]
    fn test_scxml_structural_validity() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Scxml,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");
        let scxml = adapter.export(&machine).expect("Should export to SCXML");

        // Basic structural checks
        assert!(scxml.contains("xmlns=\"http://www.w3.org/2005/07/scxml\""));
        assert!(scxml.contains("version=\"1.0\""));

        // All opening state tags should have closing tags
        let open_state_count = scxml.matches("<state ").count();
        let close_state_count = scxml.matches("</state>").count();
        assert_eq!(
            open_state_count, close_state_count,
            "State tags should be balanced"
        );

        // All transitions should be properly closed
        let open_trans_count = scxml.matches("<transition ").count();
        let self_close_trans_count = scxml.matches("/>").count();
        assert!(
            self_close_trans_count >= open_trans_count,
            "Transitions should be properly closed"
        );
    }

    #[test]
    fn test_xml_escaping() {
        // Test various XML special characters
        assert_eq!(escape_xml_attribute("a&b"), "a&amp;b");
        assert_eq!(escape_xml_attribute("a<b"), "a&lt;b");
        assert_eq!(escape_xml_attribute("a>b"), "a&gt;b");
        assert_eq!(escape_xml_attribute("a\"b"), "a&quot;b");
        assert_eq!(escape_xml_attribute("a'b"), "a&apos;b");
        assert_eq!(
            escape_xml_attribute("a&b<c>d\"e'f"),
            "a&amp;b&lt;c&gt;d&quot;e&apos;f"
        );
    }

    #[test]
    fn test_mermaid_escaping() {
        // Test Mermaid special characters
        assert_eq!(escape_mermaid("test(1)"), "test_1_");
        assert_eq!(escape_mermaid("test[1]"), "test_1_");
        assert_eq!(escape_mermaid("test{1}"), "test_1_");
        assert_eq!(escape_mermaid("test#1"), "test_1");
        assert_eq!(escape_mermaid("test:1"), "test_1");
        assert_eq!(escape_mermaid("line1\nline2"), "line1 line2");
    }

    #[test]
    fn test_completion_state() {
        let plan = create_test_plan();
        let config = XStateConfig {
            enable_visualization: true,
            export_format: ExportFormat::Json,
        };
        let adapter = XStateAdapter::new(config);

        let machine = adapter.to_xstate(&plan).expect("Should convert plan");

        // Verify completion state exists for non-empty plan with exit tasks
        let has_complete_state = machine.states.iter().any(|s| s.id == "complete");
        assert!(
            has_complete_state,
            "Should have completion state for plan with exit tasks"
        );
    }
}
