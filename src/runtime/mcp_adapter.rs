//! MCP Adapter - integrates with Model Control Protocol for tool execution

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MCP Adapter - integrates with Model Control Protocol for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPAdapter {
    /// Available tools
    tools: HashMap<String, ToolDefinition>,
    /// MCP configuration
    config: MCPConfig,
    #[serde(skip)]
    http_client: reqwest::Client,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPConfig {
    pub endpoint_url: String,
    pub api_key: Option<String>,
    pub timeout_seconds: u64,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ToolParameter>,
    pub return_type: String,
    pub category: ToolCategory,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameter {
    pub name: String,
    pub parameter_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Development,
    Testing,
    Deployment,
    Analysis,
    Security,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_name: String,
    pub arguments: HashMap<String, serde_json::Value>,
    pub execution_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub execution_id: String,
    pub success: bool,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
}

impl MCPAdapter {
    /// Create a new MCP adapter
    pub fn new(config: MCPConfig) -> Self {
        Self {
            tools: HashMap::new(),
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Register a tool
    pub fn register_tool(&mut self, tool: ToolDefinition) {
        self.tools.insert(tool.name.clone(), tool);
    }

    /// Get available tools
    pub fn list_tools(&self) -> Vec<&ToolDefinition> {
        self.tools.values().filter(|tool| tool.enabled).collect()
    }

    /// Execute a tool call
    pub async fn execute_tool(&self, tool_call: ToolCall) -> Result<ToolResponse, MCPError> {
        // Check if tool exists and is enabled
        let tool = match self.tools.get(&tool_call.tool_name) {
            Some(t) if t.enabled => t,
            Some(_) => return Err(MCPError::ToolDisabled(tool_call.tool_name)),
            None => return Err(MCPError::ToolNotFound(tool_call.tool_name)),
        };

        // Validate required parameters
        for param in &tool.parameters {
            if param.required && !tool_call.arguments.contains_key(&param.name) {
                return Err(MCPError::MissingParameter(param.name.clone()));
            }
        }

        // Call the real MCP endpoint
        let start_time = std::time::Instant::now();
        let result = self.call_mcp_endpoint(&tool_call, tool).await;
        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(ToolResponse {
            execution_id: tool_call.execution_id,
            success: result.is_ok(),
            result: result.as_ref().ok().cloned(),
            error: result.err().map(|e| e.to_string()),
            execution_time_ms: execution_time,
        })
    }

    /// Call the MCP endpoint to execute a tool
    async fn call_mcp_endpoint(
        &self,
        tool_call: &ToolCall,
        _tool: &ToolDefinition,
    ) -> Result<serde_json::Value, MCPError> {
        let mcp_url = format!(
            "{}/tools/{}",
            self.config.endpoint_url.trim_end_matches('/'),
            tool_call.tool_name
        );

        let mut request = self
            .http_client
            .post(&mcp_url)
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(self.config.timeout_seconds));

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.json(&tool_call.arguments).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(MCPError::ExecutionFailed(format!(
                "MCP API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_json: serde_json::Value = response.json().await?;
        Ok(response_json)
    }

    /// Get tool by name
    pub fn get_tool(&self, name: &str) -> Option<&ToolDefinition> {
        self.tools.get(name)
    }

    /// Get configuration
    pub fn config(&self) -> &MCPConfig {
        &self.config
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MCPError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    #[error("Tool disabled: {0}")]
    ToolDisabled(String),
    #[error("Missing required parameter: {0}")]
    MissingParameter(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    #[error("Authentication error: {0}")]
    AuthError(String),
    #[error("Tool execution failed: {0}")]
    ExecutionFailed(String),
}

impl Default for MCPAdapter {
    fn default() -> Self {
        Self::new(MCPConfig {
            endpoint_url: "http://localhost:8000".to_string(),
            api_key: None,
            timeout_seconds: 30,
            max_retries: 3,
        })
    }
}

// Predefined common tools
impl MCPAdapter {
    /// Register common development tools
    pub fn register_common_tools(&mut self) {
        // Code analyzer tool
        self.register_tool(ToolDefinition {
            name: "code_analyzer".to_string(),
            description: "Analyzes code quality and suggests improvements".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "path".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Path to the code to analyze".to_string(),
                    required: true,
                    default_value: None,
                },
                ToolParameter {
                    name: "ruleset".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Quality ruleset to apply".to_string(),
                    required: false,
                    default_value: Some(serde_json::Value::String("default".to_string())),
                },
            ],
            return_type: "object".to_string(),
            category: ToolCategory::Development,
            enabled: true,
        });

        // Test runner tool
        self.register_tool(ToolDefinition {
            name: "test_runner".to_string(),
            description: "Runs automated tests and reports results".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "test_suite".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Name of the test suite to run".to_string(),
                    required: true,
                    default_value: None,
                },
                ToolParameter {
                    name: "parallel".to_string(),
                    parameter_type: "boolean".to_string(),
                    description: "Whether to run tests in parallel".to_string(),
                    required: false,
                    default_value: Some(serde_json::Value::Bool(true)),
                },
            ],
            return_type: "object".to_string(),
            category: ToolCategory::Testing,
            enabled: true,
        });

        // Deploy tool
        self.register_tool(ToolDefinition {
            name: "deploy_tool".to_string(),
            description: "Deploys applications to target environments".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "environment".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Target environment (dev, staging, prod)".to_string(),
                    required: true,
                    default_value: None,
                },
                ToolParameter {
                    name: "version".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Version to deploy".to_string(),
                    required: true,
                    default_value: None,
                },
            ],
            return_type: "object".to_string(),
            category: ToolCategory::Deployment,
            enabled: true,
        });

        // Security scanner tool
        self.register_tool(ToolDefinition {
            name: "security_scanner".to_string(),
            description: "Scans for security vulnerabilities".to_string(),
            parameters: vec![
                ToolParameter {
                    name: "target".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Target to scan (code, dependencies, infrastructure)".to_string(),
                    required: true,
                    default_value: None,
                },
                ToolParameter {
                    name: "depth".to_string(),
                    parameter_type: "string".to_string(),
                    description: "Scan depth (quick, thorough)".to_string(),
                    required: false,
                    default_value: Some(serde_json::Value::String("thorough".to_string())),
                },
            ],
            return_type: "object".to_string(),
            category: ToolCategory::Security,
            enabled: true,
        });
    }
}
