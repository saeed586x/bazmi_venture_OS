//! Adapters for integrating with external systems

pub mod llm_adapter;
pub mod mcp_adapter;
pub mod observability;
pub mod restate_adapter;
pub mod xstate_adapter;

pub use llm_adapter::LLMAdapter;
pub use mcp_adapter::MCPAdapter;
pub use observability::Observability;
pub use restate_adapter::RestateAdapter;
pub use xstate_adapter::XStateAdapter;
