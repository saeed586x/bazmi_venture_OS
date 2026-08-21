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
        // Build states from tasks
        let mut states = Vec::new();
        let mut transitions = Vec::new();

        // Add initial state
        states.push(XState {
            id: "start".to_string(),
            name: "Start".to_string(),
            type_: StateType::Atomic,
        });

        // Add task states
        for task in &plan.tasks {
            states.push(XState {
                id: task.id.clone(),
                name: task.name.clone(),
                type_: StateType::Atomic,
            });
        }

        // Add gate states
        for gate in &plan.gates {
            states.push(XState {
                id: format!("gate_{}", gate.id),
                name: gate.name.clone(),
                type_: StateType::Atomic,
            });
        }

        // Add final/completion states
        for condition in &plan.completion_conditions {
            states.push(XState {
                id: format!("completion_{}", condition.id),
                name: format!("Check: {}", condition.description),
                type_: StateType::Final,
            });
        }

        // Build transitions from dependencies
        for dep in &plan.dependencies {
            transitions.push(XTransition {
                from: dep.dependency_task_id.clone(),
                to: dep.dependent_task_id.clone(),
                event: "completed".to_string(),
                guard: None,
            });
        }

        // Add transition from start to first task
        if let Some(first_task) = plan.tasks.first() {
            transitions.push(XTransition {
                from: "start".to_string(),
                to: first_task.id.clone(),
                event: "init".to_string(),
                guard: None,
            });
        }

        // Add transitions to completion states
        if let Some(last_task) = plan.tasks.last() {
            for condition in &plan.completion_conditions {
                transitions.push(XTransition {
                    from: last_task.id.clone(),
                    to: format!("completion_{}", condition.id),
                    event: "task_completed".to_string(),
                    guard: Some(format!("evaluate({})", condition.expression)),
                });
            }
        }

        Ok(XStateMachine {
            id: format!("plan_{}", plan.id),
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

    /// Export to SCXML format
    fn export_scxml(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        let mut scxml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        scxml.push_str(&format!(
            "<scxml xmlns=\"http://www.w3.org/2005/07/scxml\" version=\"1.0\" initial=\"{}\">\n",
            machine.initial
        ));

        for state in &machine.states {
            scxml.push_str(&format!(
                "  <state id=\"{}\" name=\"{}\">\n",
                state.id, state.name
            ));
            scxml.push_str("  </state>\n");
        }

        for transition in &machine.transitions {
            scxml.push_str(&format!(
                "  <transition from=\"{}\" to=\"{}\" event=\"{}\"",
                transition.from, transition.to, transition.event
            ));
            if let Some(guard) = &transition.guard {
                scxml.push_str(&format!(" cond=\"{}\"", guard));
            }
            scxml.push_str(
                "/>
",
            );
        }

        scxml.push_str("</scxml>");
        Ok(scxml)
    }

    /// Export to Mermaid state diagram format
    fn export_mermaid(&self, machine: &XStateMachine) -> Result<String, XStateError> {
        let mut mermaid = String::from("stateDiagram-v2\n");
        mermaid.push_str(&format!("    [*] --> {}\n", machine.initial));

        for transition in &machine.transitions {
            let label = if let Some(guard) = &transition.guard {
                format!("{} [{}] ", transition.event, guard)
            } else {
                transition.event.clone()
            };
            mermaid.push_str(&format!(
                "    {} --> {}: {}\n",
                transition.from, transition.to, label
            ));
        }

        // Mark final states
        for state in &machine.states {
            if matches!(state.type_, StateType::Final) {
                mermaid.push_str(&format!("    {} --> [*]\n", state.id));
            }
        }

        Ok(mermaid)
    }
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
}
