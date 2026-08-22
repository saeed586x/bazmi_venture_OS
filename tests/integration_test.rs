//! Integration tests for the Venture OS Kernel

use venture_os_kernel::contracts::ExecutionPlanV1;
use venture_os_kernel::core::governance::Governance;
use venture_os_kernel::core::semantic_model::SemanticModel;
use venture_os_kernel::Kernel;

// Import capabilities directly from their modules
use venture_os_kernel::capabilities::ard_compiler::ARDCompiler;
use venture_os_kernel::capabilities::context_engine::ContextEngine;
use venture_os_kernel::capabilities::decision_memory::DecisionMemory;
use venture_os_kernel::capabilities::domain_engine::DomainEngine;
use venture_os_kernel::capabilities::intent_engine::IntentEngine;
use venture_os_kernel::capabilities::planning_engine::{PlanningContext, PlanningEngine};
use venture_os_kernel::capabilities::prd_compiler::PRDCompiler;
use venture_os_kernel::capabilities::requirements_engine::RequirementsEngine;
use venture_os_kernel::capabilities::risk_engine::RiskEngine;
use venture_os_kernel::capabilities::validation_engine::ValidationEngine;

#[test]
fn test_kernel_creation() {
    let _kernel = Kernel::new();
    // Basic test to ensure kernel creation works
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
    // Basic test
}

#[test]
fn test_requirements_engine() {
    let _requirements_engine = RequirementsEngine::new();
    // Basic test
}

#[test]
fn test_context_engine() {
    let context_engine = ContextEngine::new();

    // Test initial context
    let context = context_engine.get_current_context();
    assert!(context.resources.is_empty());

    // Test context analysis
    let analysis = context_engine.analyze_context();
    assert!(analysis.risk_factors.is_empty());

    // Basic functionality test
}

#[test]
fn test_prd_compiler() {
    let semantic_model = SemanticModel::new();
    let prd_compiler = PRDCompiler::new(semantic_model);

    let intent = "Build a customer portal";
    let prd = prd_compiler.compile_from_intent(intent);
    assert!(prd.is_ok());

    let prd_doc = prd.unwrap();
    assert!(!prd_doc.id.is_empty());
    assert!(prd_doc.title.contains(intent));
}

#[test]
fn test_ard_compiler() {
    let semantic_model = SemanticModel::new();
    let ard_compiler = ARDCompiler::new(semantic_model);

    // Create a simple domain model for testing
    let domain_model = venture_os_kernel::capabilities::domain_engine::DomainModel {
        id: "test-model".to_string(),
        name: "Test Model".to_string(),
        description: "A test domain model".to_string(),
        version: "1.0.0".to_string(),
        entities: vec![],
        relationships: vec![],
    };

    let ard = ard_compiler.compile_from_domain_model(&domain_model);
    assert!(!ard.id.is_empty());
    assert_eq!(ard.title, "ARD for Test Model");
}

#[test]
fn test_validation_engine() {
    let governance = Governance::new();
    let validation_engine = ValidationEngine::new(governance);

    // Create a minimal valid plan for testing
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

    let validation_result = validation_engine.validate_execution_plan(&plan);
    assert!(validation_result.valid);
}

#[test]
fn test_risk_engine() {
    let context_engine = ContextEngine::new();
    let risk_engine = RiskEngine::new(context_engine);

    // Create a minimal plan for testing
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

    let risk_assessment = risk_engine.assess_execution_plan_risks(&plan);
    assert!(!risk_assessment.id.is_empty());
    assert_eq!(risk_assessment.assessed_item, plan.id);
}

#[test]
fn test_decision_memory() {
    let decision_memory = DecisionMemory::new();

    // Basic functionality test
    let stats = decision_memory.get_statistics();
    assert_eq!(stats.total_decisions, 0);

    // Test pattern learning
    let patterns = decision_memory.get_patterns();
    assert_eq!(patterns.len(), 0);
}

#[test]
fn test_planning_engine() {
    let context_engine = ContextEngine::new();
    let planning_engine = PlanningEngine::new(context_engine);

    // Create a simple planning context
    let context = PlanningContext {
        goals: vec![],
        constraints: vec![],
        available_capabilities: vec!["development".to_string()],
        resource_limits: std::collections::HashMap::new(),
        deadline: None,
        priority: 5,
    };

    let planned_execution = planning_engine.create_plan(context);
    assert!(planned_execution.is_ok());

    let execution = planned_execution.unwrap();
    assert!(!execution.plan.id.is_empty());
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
