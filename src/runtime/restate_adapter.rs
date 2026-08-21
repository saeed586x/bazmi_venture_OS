//! Restate adapter for durable execution

use crate::contracts::ExecutionPlanV1;
use serde::{Deserialize, Serialize};

/// Restate adapter for durable execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestateAdapter {
    /// Configuration for the Restate adapter
    config: RestateConfig,
    #[serde(skip)]
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestateConfig {
    pub endpoint_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
}

impl RestateAdapter {
    /// Create a new Restate adapter
    pub fn new(config: RestateConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Submit an execution plan to Restate for durable execution
    pub async fn submit_plan(
        &self,
        plan: &ExecutionPlanV1,
    ) -> Result<ExecutionResult, RestateError> {
        let submit_url = format!("{}/invoke", self.config.endpoint_url.trim_end_matches('/'));

        // Convert execution plan to Restate-compatible workflow invocation
        let workflow_request = serde_json::json!({
            "workflow_id": format!("plan_{}", plan.id),
            "input": {
                "execution_plan": plan,
                "metadata": {
                    "created_at": chrono::Utc::now().to_rfc3339(),
                    "version": "v1"
                }
            }
        });

        let mut request = self
            .http_client
            .post(&submit_url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds));

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(&workflow_request)
            .send()
            .await
            .map_err(|e| RestateError::NetworkError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(RestateError::ExecutionFailed(format!(
                "Restate API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| RestateError::NetworkError(format!("JSON parse failed: {}", e)))?;

        // Parse Restate response to get execution ID and status
        let execution_id = response_json
            .pointer("/execution_id")
            .and_then(|v| v.as_str())
            .unwrap_or(&format!("exec_{}", plan.id))
            .to_string();

        let status = response_json
            .pointer("/status")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "running" => ExecutionStatus::Running,
                "completed" => ExecutionStatus::Completed,
                "failed" => ExecutionStatus::Failed,
                "cancelled" => ExecutionStatus::Cancelled,
                _ => ExecutionStatus::Submitted,
            })
            .unwrap_or(ExecutionStatus::Submitted);

        Ok(ExecutionResult {
            execution_id,
            status,
            result_data: Some(response_json),
        })
    }

    /// Check the status of an execution
    pub async fn get_execution_status(
        &self,
        execution_id: &str,
    ) -> Result<ExecutionStatus, RestateError> {
        let status_url = format!(
            "{}/status/{}",
            self.config.endpoint_url.trim_end_matches('/'),
            execution_id
        );

        let mut request = self
            .http_client
            .get(&status_url)
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds));

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .send()
            .await
            .map_err(|e| RestateError::NetworkError(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(RestateError::ExecutionFailed(format!(
                "Restate status request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| RestateError::NetworkError(format!("JSON parse failed: {}", e)))?;

        let status = response_json
            .pointer("/status")
            .and_then(|v| v.as_str())
            .map(|s| match s {
                "running" => ExecutionStatus::Running,
                "completed" => ExecutionStatus::Completed,
                "failed" => ExecutionStatus::Failed,
                "cancelled" => ExecutionStatus::Cancelled,
                _ => ExecutionStatus::Submitted,
            })
            .unwrap_or(ExecutionStatus::Running);

        Ok(status)
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
}
