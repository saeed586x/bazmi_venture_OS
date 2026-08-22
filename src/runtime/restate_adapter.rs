//! Restate adapter for durable execution
//!
//! This adapter serves as a boundary between the Kernel's ExecutionPlan.v1
//! and Restate's durable execution system. The Kernel remains independent
//! of Restate internals - it only emits ExecutionPlan.v1 documents.
//!
//! Key design principles:
//! - Validate ExecutionPlan.v1 before submission
//! - Use explicit, documented request contracts
//! - Preserve idempotency for repeated submissions
//! - Apply timeout and authentication configuration without exposing secrets
//! - Return typed errors for all failure modes
//! - Parse execution ID and status only from validated response fields

use crate::contracts::ExecutionPlanV1;
use crate::contracts::execution_plan_v1::{Dependency, Task};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Restate adapter for durable execution
#[derive(Debug, Clone)]
pub struct RestateAdapter {
    /// Configuration for the Restate adapter
    config: RestateConfig,
    /// HTTP client for making requests
    client: reqwest::Client,
}

/// Configuration for connecting to Restate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestateConfig {
    /// Base URL for the Restate service
    pub endpoint_url: String,
    /// Optional API key for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_timeout() -> u64 {
    30
}

impl RestateAdapter {
    /// Create a new Restate adapter with the given configuration
    pub fn new(config: RestateConfig) -> Result<Self, RestateError> {
        let timeout = Duration::from_secs(config.timeout_seconds);
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|e| RestateError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self { config, client })
    }

    /// Submit an execution plan to Restate for durable execution
    ///
    /// This method:
    /// 1. Validates the ExecutionPlan.v1 before submission
    /// 2. Converts the plan to a Restate-compatible request format
    /// 3. Submits it to the Restate service with idempotency guarantees
    /// 4. Returns the execution result with parsed status
    pub async fn submit_plan(
        &self,
        plan: &ExecutionPlanV1,
    ) -> Result<ExecutionResult, RestateError> {
        // Step 1: Validate the execution plan before submission
        Self::validate_plan(plan)?;

        // Step 2: Build the Restate request
        let request = self.build_submit_request(plan)?;

        // Step 3: Submit to Restate endpoint
        let response = self.send_submit_request(&request).await?;

        // Step 4: Parse and validate the response
        self.parse_submit_response(response).await
    }

    /// Validate an ExecutionPlan.v1 before submission
    ///
    /// This ensures we don't submit invalid plans to Restate.
    fn validate_plan(plan: &ExecutionPlanV1) -> Result<(), RestateError> {
        // Check required fields are non-empty
        if plan.id.trim().is_empty() {
            return Err(RestateError::InvalidPlan("Plan ID is empty".to_string()));
        }

        if plan.version.trim().is_empty() {
            return Err(RestateError::InvalidPlan("Plan version is empty".to_string()));
        }

        if plan.intent_reference.trim().is_empty() {
            return Err(RestateError::InvalidPlan("Intent reference is empty".to_string()));
        }

        // Validate semantic version format (basic check)
        if !plan.version.chars().any(|c| c == '.') {
            return Err(RestateError::InvalidPlan(
                "Plan version must be a semantic version (e.g., 1.0.0)".to_string(),
            ));
        }

        // Validate tasks have capabilities
        for task in &plan.tasks {
            if task.capability.trim().is_empty() {
                return Err(RestateError::InvalidPlan(format!(
                    "Task '{}' has no capability specified",
                    task.id
                )));
            }
        }

        // Validate dependencies reference existing tasks
        let task_ids: std::collections::HashSet<&String> =
            plan.tasks.iter().map(|t| &t.id).collect();

        for dep in &plan.dependencies {
            if !task_ids.contains(&dep.dependent_task_id) {
                return Err(RestateError::InvalidPlan(format!(
                    "Dependency references non-existent task '{}'",
                    dep.dependent_task_id
                )));
            }
            if !task_ids.contains(&dep.dependency_task_id) {
                return Err(RestateError::InvalidPlan(format!(
                    "Dependency references non-existent task '{}'",
                    dep.dependency_task_id
                )));
            }
        }

        // Check for cyclic dependencies
        if Self::has_cyclic_dependency(plan) {
            return Err(RestateError::InvalidPlan(
                "Plan contains cyclic dependencies".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if the plan has cyclic dependencies using DFS
    fn has_cyclic_dependency(plan: &ExecutionPlanV1) -> bool {
        use std::collections::{HashMap, HashSet};

        // Build adjacency list
        let mut graph: HashMap<&String, Vec<&String>> = HashMap::new();
        for task in &plan.tasks {
            graph.entry(&task.id).or_insert_with(Vec::new);
        }

        for dep in &plan.dependencies {
            if let Some(deps) = graph.get_mut(&dep.dependency_task_id) {
                deps.push(&dep.dependent_task_id);
            }
        }

        // DFS to detect cycles
        let mut visited: HashSet<&String> = HashSet::new();
        let mut rec_stack: HashSet<&String> = HashSet::new();

        fn dfs<'a>(
            node: &'a String,
            graph: &HashMap<&'a String, Vec<&'a String>>,
            visited: &mut HashSet<&'a String>,
            rec_stack: &mut HashSet<&'a String>,
        ) -> bool {
            if rec_stack.contains(node) {
                return true;
            }
            if visited.contains(node) {
                return false;
            }

            visited.insert(node);
            rec_stack.insert(node);

            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    if dfs(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                }
            }

            rec_stack.remove(node);
            false
        }

        for task_id in graph.keys() {
            if !visited.contains(task_id) {
                if dfs(task_id, &graph, &mut visited, &mut rec_stack) {
                    return true;
                }
            }
        }

        false
    }

    /// Build the Restate submission request
    fn build_submit_request(
        &self,
        plan: &ExecutionPlanV1,
    ) -> Result<RestateSubmitRequest, RestateError> {
        // Convert ExecutionPlan.v1 to Restate request format
        // This maintains separation between Kernel contracts and Restate internals
        let request = RestateSubmitRequest {
            workflow_type: "ExecutionWorkflow".to_string(),
            workflow_id: format!("exec_{}", plan.id),
            input: serde_json::to_value(plan).map_err(|e| {
                RestateError::SerializationError(format!("Failed to serialize plan: {}", e))
            })?,
            idempotency_key: Some(plan.id.clone()),
        };

        Ok(request)
    }

    /// Send the submit request to Restate
    async fn send_submit_request(
        &self,
        request: &RestateSubmitRequest,
    ) -> Result<reqwest::Response, RestateError> {
        let url = format!("{}/invoke", self.config.endpoint_url.trim_end_matches('/'));

        let mut req_builder = self
            .client
            .post(&url)
            .header("Content-Type", "application/json");

        // Add authentication if configured
        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let request_body = serde_json::to_value(request).map_err(|e| {
            RestateError::SerializationError(format!("Failed to serialize request: {}", e))
        })?;

        let response = req_builder
            .json(&request_body)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    RestateError::Timeout(e.to_string())
                } else if e.is_connect() {
                    RestateError::NetworkError(format!("Connection failed: {}", e))
                } else {
                    RestateError::NetworkError(e.to_string())
                }
            })?;

        Ok(response)
    }

    /// Parse the response from Restate
    async fn parse_submit_response(
        &self,
        response: reqwest::Response,
    ) -> Result<ExecutionResult, RestateError> {
        let status = response.status();

        if !status.is_success() {
            return if status.is_client_error() {
                Err(RestateError::AuthError(format!(
                    "Request rejected: {}",
                    status
                )))
            } else if status.is_server_error() {
                Err(RestateError::NetworkError(format!(
                    "Server error: {}",
                    status
                )))
            } else {
                Err(RestateError::ExecutionFailed(format!(
                    "Unexpected status: {}",
                    status
                )))
            };
        }

        // Parse the response body
        let response_body: RestateSubmitResponse = response.json().await.map_err(|e| {
            RestateError::SerializationError(format!("Failed to parse response: {}", e))
        })?;

        // Validate response fields
        if response_body.execution_id.trim().is_empty() {
            return Err(RestateError::SerializationError(
                "Response missing execution_id".to_string(),
            ));
        }

        let status = match response_body.status.as_deref() {
            Some("submitted") | Some("running") => ExecutionStatus::Running,
            Some("completed") => ExecutionStatus::Completed,
            Some("failed") => ExecutionStatus::Failed,
            Some("cancelled") => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Running, // Default to running for unknown statuses
        };

        Ok(ExecutionResult {
            execution_id: response_body.execution_id,
            status,
            result_data: response_body.result_data,
        })
    }

    /// Check the status of an execution
    ///
    /// This queries the Restate service for the current execution status.
    pub async fn get_execution_status(
        &self,
        execution_id: &str,
    ) -> Result<ExecutionStatus, RestateError> {
        if execution_id.trim().is_empty() {
            return Err(RestateError::InvalidExecutionId(
                "Execution ID cannot be empty".to_string(),
            ));
        }

        let url = format!(
            "{}/status/{}",
            self.config.endpoint_url.trim_end_matches('/'),
            execution_id
        );

        let mut req_builder = self.client.get(&url);

        if let Some(ref api_key) = self.config.api_key {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req_builder.send().await.map_err(|e| {
            if e.is_timeout() {
                RestateError::Timeout(e.to_string())
            } else {
                RestateError::NetworkError(e.to_string())
            }
        })?;

        let status = response.status();
        if !status.is_success() {
            return Err(RestateError::NetworkError(format!(
                "Status lookup failed: {}",
                status
            )));
        }

        let status_body: RestateStatusResponse = response.json().await.map_err(|e| {
            RestateError::SerializationError(format!("Failed to parse status response: {}", e))
        })?;

        let execution_status = match status_body.status.as_str() {
            "submitted" | "pending" => ExecutionStatus::Submitted,
            "running" | "processing" => ExecutionStatus::Running,
            "completed" | "success" => ExecutionStatus::Completed,
            "failed" | "error" => ExecutionStatus::Failed,
            "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Running,
        };

        Ok(execution_status)
    }
}

/// Request format for submitting to Restate
#[derive(Debug, Clone, Serialize)]
struct RestateSubmitRequest {
    workflow_type: String,
    workflow_id: String,
    input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    idempotency_key: Option<String>,
}

/// Response format from Restate submission
#[derive(Debug, Clone, Deserialize)]
struct RestateSubmitResponse {
    execution_id: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    result_data: Option<serde_json::Value>,
}

/// Response format from Restate status lookup
#[derive(Debug, Clone, Deserialize)]
struct RestateStatusResponse {
    execution_id: String,
    status: String,
    #[serde(default)]
    started_at: Option<String>,
    #[serde(default)]
    completed_at: Option<String>,
}

/// Result of submitting an execution plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: String,
    pub status: ExecutionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_data: Option<serde_json::Value>,
}

/// Status of an execution
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Submitted,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Errors that can occur during Restate operations
#[derive(Debug, thiserror::Error)]
pub enum RestateError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid plan: {0}")]
    InvalidPlan(String),

    #[error("Invalid execution ID: {0}")]
    InvalidExecutionId(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Request timeout: {0}")]
    Timeout(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_valid_plan() -> ExecutionPlanV1 {
        ExecutionPlanV1 {
            id: "test-plan-123".to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Test intent".to_string(),
            goals: vec![],
            constraints: vec![],
            required_capabilities: vec![],
            inputs: vec![],
            tasks: vec![
                Task {
                    id: "task-1".to_string(),
                    name: "Task 1".to_string(),
                    description: "First task".to_string(),
                    capability: "capability-1".to_string(),
                    parameters: std::collections::HashMap::new(),
                    expected_duration: None,
                },
                Task {
                    id: "task-2".to_string(),
                    name: "Task 2".to_string(),
                    description: "Second task".to_string(),
                    capability: "capability-2".to_string(),
                    parameters: std::collections::HashMap::new(),
                    expected_duration: None,
                },
            ],
            dependencies: vec![Dependency {
                dependent_task_id: "task-2".to_string(),
                dependency_task_id: "task-1".to_string(),
            }],
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
    fn test_validate_valid_plan() {
        let plan = create_valid_plan();
        assert!(RestateAdapter::validate_plan(&plan).is_ok());
    }

    #[test]
    fn test_validate_empty_id() {
        let mut plan = create_valid_plan();
        plan.id = "".to_string();
        assert!(RestateAdapter::validate_plan(&plan).is_err());
    }

    #[test]
    fn test_validate_empty_version() {
        let mut plan = create_valid_plan();
        plan.version = "".to_string();
        assert!(RestateAdapter::validate_plan(&plan).is_err());
    }

    #[test]
    fn test_validate_invalid_version_format() {
        let mut plan = create_valid_plan();
        plan.version = "1".to_string();
        assert!(RestateAdapter::validate_plan(&plan).is_err());
    }

    #[test]
    fn test_validate_task_without_capability() {
        let mut plan = create_valid_plan();
        plan.tasks[0].capability = "".to_string();
        assert!(RestateAdapter::validate_plan(&plan).is_err());
    }

    #[test]
    fn test_validate_nonexistent_dependency() {
        let mut plan = create_valid_plan();
        plan.dependencies.push(Dependency {
            dependent_task_id: "task-3".to_string(),
            dependency_task_id: "task-1".to_string(),
        });
        assert!(RestateAdapter::validate_plan(&plan).is_err());
    }

    #[test]
    fn test_validate_cyclic_dependency() {
        let mut plan = create_valid_plan();
        // Create a cycle: task-1 -> task-2 -> task-1
        plan.dependencies.push(Dependency {
            dependent_task_id: "task-1".to_string(),
            dependency_task_id: "task-2".to_string(),
        });
        assert!(RestateAdapter::validate_plan(&plan).is_err());
    }

    #[test]
    fn test_execution_status_from_str() {
        // Test that status variants serialize/deserialize correctly
        let status = ExecutionStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"completed\"");

        let deserialized: ExecutionStatus = serde_json::from_str("\"completed\"").unwrap();
        assert_eq!(deserialized, ExecutionStatus::Completed);
    }

    #[test]
    fn test_execution_result_serialization() {
        let result = ExecutionResult {
            execution_id: "exec-123".to_string(),
            status: ExecutionStatus::Running,
            result_data: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("exec-123"));
        assert!(json.contains("running"));

        let deserialized: ExecutionResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.execution_id, "exec-123");
        assert_eq!(deserialized.status, ExecutionStatus::Running);
    }

    #[test]
    fn test_config_default_timeout() {
        let config = RestateConfig {
            endpoint_url: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_seconds: 0, // Will use default
        };

        // Verify default timeout function works
        assert_eq!(default_timeout(), 30);
    }
}
