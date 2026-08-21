//! Adapters for integrating with external systems

pub mod llm_adapter;
pub mod restate_adapter;
pub mod xstate_adapter;

pub use llm_adapter::LLMAdapter;
pub use restate_adapter::RestateAdapter;
pub use xstate_adapter::XStateAdapter;
