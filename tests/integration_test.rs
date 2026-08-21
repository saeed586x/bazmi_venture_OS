//! Integration tests for the Venture OS Kernel

use venture_os_kernel::Kernel;
use venture_os_kernel::capabilities::domain_engine::DomainEngine;
use venture_os_kernel::capabilities::intent_engine::IntentEngine;
use venture_os_kernel::capabilities::requirements_engine::RequirementsEngine;
use venture_os_kernel::contracts::ExecutionPlanV1;

#[test]
fn test_kernel_creation() {
    let _kernel = Kernel::new();
    // Basic test to ensure kernel creation works - removed trivial assertion
}

#[test]
fn test_intent_processing() {
    let intent_engine = IntentEngine::default();
    let intent = "Create a new customer management system";

    let processed = intent_engine.process_intent(intent);
    assert!(processed.is_ok());

    let execution_plan = intent_engine.create_execution_plan(&processed.unwrap());
    assert!(!execution_plan.id.is_empty());
    assert_eq!(execution_plan.intent_reference, intent);
}

#[test]
fn test_domain_engine() {
    let _domain_engine = DomainEngine::new();
    // Basic test - removed trivial assertion
}

#[test]
fn test_requirements_engine() {
    let _requirements_engine = RequirementsEngine::new();
    // Basic test - removed trivial assertion
}

#[test]
fn test_execution_plan_contract() {
    let plan = ExecutionPlanV1 {
        id: "test-plan-123".to_string(),
        version: "1.0.0".to_string(),
        parent_plan_id: None,
        intent_reference: "Test intent".to_string(),
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
        creation_timestamp: chrono::Utc::now(),
        replan_reason: None,
    };

    assert_eq!(plan.id, "test-plan-123");
    assert_eq!(plan.version, "1.0.0");
}
