//! Restate adapter for durable execution

use crate::contracts::{ExecutionPlanV1, PlanValidationError};
use serde::{Deserialize, Serialize};

/// Restate adapter for durable execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestateAdapter {
    /// Configuration for the Restate adapter
    config: RestateConfig,
    /// Whether to validate plans before submission
    validate_plans: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestateConfig {
    pub endpoint_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    /// Whether to validate execution plans before submission
    #[serde(default = "default_validate_plans")]
    pub validate_plans: bool,
}

fn default_validate_plans() -> bool {
    true
}

impl RestateAdapter {
    /// Create a new Restate adapter
    pub fn new(config: RestateConfig) -> Self {
        Self { 
            validate_plans: config.validate_plans,
            config,
        }
    }

    /// Create a new Restate adapter with default configuration
    pub fn with_defaults() -> Self {
        Self::new(RestateConfig {
            endpoint_url: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_seconds: 30,
            validate_plans: true,
        })
    }

    /// Submit an execution plan to Restate for durable execution
    pub async fn submit_plan(
        &self,
        plan: &ExecutionPlanV1,
    ) -> Result<ExecutionResult, RestateError> {
        // Validate the plan before submission if validation is enabled
        if self.validate_plans {
            self.validate_plan(plan)?;
        }

        // In a real implementation, this would:
        // 1. Convert the execution plan to Restate-compatible format
        // 2. Submit it to the Restate service
        // 3. Return the execution result

        Ok(ExecutionResult {
            execution_id: format!("exec_{}", plan.id),
            status: ExecutionStatus::Submitted,
            result_data: None,
        })
    }

    /// Validate an execution plan before submission
    fn validate_plan(&self, plan: &ExecutionPlanV1) -> Result<(), RestateError> {
        plan.validate().map_err(|e| RestateError::ValidationError(e.0))
    }

    /// Check the status of an execution
    pub async fn get_execution_status(
        &self,
        _execution_id: &str,
    ) -> Result<ExecutionStatus, RestateError> {
        // In a real implementation, this would query the Restate service
        // for the execution status

        Ok(ExecutionStatus::Running)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub result_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    Submitted,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum RestateError {
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    #[error("Plan validation failed: {0}")]
    ValidationError(String),
}
