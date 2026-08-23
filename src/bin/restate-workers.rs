//! Restate Workers for Venture OS Execution Layer
//!
//! This module contains placeholder implementations for Restate workers.
//! The restate-sdk API has evolved and these traits need to be updated to match
//! the current SDK version. For now, this file is excluded from builds with --all-features
//! unless the restate-sdk compatibility is restored.

// Temporarily disabled due to restate-sdk API changes
// The following workers need to be updated to use the current restate_sdk::prelude::HandlerResult
// and single-argument handler signatures:
//
// - LlmWorker: async fn generate_response(prompt: String) -> HandlerResult<String>
// - ToolWorker: async fn execute_tool(input: ToolInput) -> HandlerResult<Value>
// - GateWorker: async fn evaluate_rule(input: RuleInput) -> HandlerResult<bool>
// - ExecutionWorkflow: async fn process_plan(plan_json: String) -> HandlerResult<String>
//
// See ISSUE-08 for classification of production stubs.
// This file is intentionally excluded from --all-features builds until updated.

fn main() {
    // Placeholder main to satisfy binary requirement
    // Real implementation requires updating to restate-sdk 0.2 API
    eprintln!("restate-workers binary is a placeholder - see ISSUE-08");
}
