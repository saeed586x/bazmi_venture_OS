//! End-to-end integration tests for the Venture OS Kernel

use venture_os_kernel::contracts::ExecutionPlanV1;
use venture_os_kernel::core::{Governance, SemanticModel};

// Import capabilities directly from their modules
use venture_os_kernel::capabilities::ard_compiler::ARDCompiler;
use venture_os_kernel::capabilities::context_engine::ContextEngine;
use venture_os_kernel::capabilities::decision_gateway::DecisionGateway;
use venture_os_kernel::capabilities::domain_engine::DomainEngine;
use venture_os_kernel::capabilities::intent_engine::IntentEngine;
use venture_os_kernel::capabilities::planning_engine::{PlanningContext, PlanningEngine};
use venture_os_kernel::capabilities::prd_compiler::PRDCompiler;
use venture_os_kernel::capabilities::requirements_engine::RequirementsEngine;
use venture_os_kernel::capabilities::risk_engine::RiskEngine;
use venture_os_kernel::capabilities::validation_engine::ValidationEngine;

#[test]
fn test_complete_e2e_workflow() {
    // Step 1: Start with an idea/intent
    let intent = "Build a customer portal for managing orders and invoices";

    // Step 2: Process intent
    let intent_engine = IntentEngine::default();
    let processed_intent = intent_engine
        .process_intent(intent)
        .expect("Intent processing should succeed");

    // Step 3: Domain modeling
    let _domain_engine = DomainEngine::new();
    // In a real scenario, we'd populate the domain model

    // Step 4: Requirements engineering
    let _requirements_engine = RequirementsEngine::new();
    // In a real scenario, we'd define requirements

    // Step 5: Context analysis
    let context_engine = ContextEngine::new();
    let _context_analysis = context_engine.analyze_context();

    // Step 6: PRD compilation
    let semantic_model = SemanticModel::new();
    let prd_compiler = PRDCompiler::new(semantic_model);
    let prd = prd_compiler
        .compile_from_intent(intent)
        .expect("PRD compilation should succeed");

    // Step 7: ARD compilation
    let ard_semantic_model = SemanticModel::new();
    let ard_compiler = ARDCompiler::new(ard_semantic_model);

    // Create a simple domain model for ARD
    let domain_model = venture_os_kernel::capabilities::domain_engine::DomainModel {
        id: "customer-portal".to_string(),
        name: "Customer Portal".to_string(),
        description: "Portal for customer self-service".to_string(),
        version: "1.0.0".to_string(),
        entities: vec![],
        relationships: vec![],
    };

    let ard = ard_compiler.compile_from_domain_model(&domain_model);

    // Step 8: Validation
    let governance = Governance::new();
    let validation_engine = ValidationEngine::new(governance);

    // Create a plan to validate
    let plan = ExecutionPlanV1 {
        id: "test-plan-123".to_string(),
        version: "1.0.0".to_string(),
        parent_plan_id: None,
        intent_reference: intent.to_string(),
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

    // Step 9: Risk assessment
    let risk_engine = RiskEngine::new(ContextEngine::new());
    let risk_assessment = risk_engine.assess_execution_plan_risks(&plan);

    // Step 10: Decision gateway
    let _decision_gateway = DecisionGateway::new(
        ValidationEngine::default(),
        venture_os_kernel::capabilities::verification_engine::VerificationEngine::default(),
        RiskEngine::default(),
    );

    let _decision_context = venture_os_kernel::capabilities::decision_gateway::DecisionContext {
        plan: plan.clone(),
        validation_result: Some(validation_result.clone()),
        verification_result: None,
        risk_assessment: Some(risk_assessment.clone()),
        metadata: std::collections::HashMap::new(),
    };

    // This would be async in real usage
    // let decision_outcome = decision_gateway.process_decision(decision_context).await;

    // Step 11: Planning
    let planning_engine = PlanningEngine::new(ContextEngine::new());
    let planning_context = PlanningContext {
        goals: vec![],
        constraints: vec![],
        available_capabilities: vec!["web-development".to_string(), "database".to_string()],
        resource_limits: std::collections::HashMap::new(),
        deadline: None,
        priority: 5,
    };

    let planned_execution = planning_engine.create_plan(planning_context);

    // Assertions - verify all steps completed successfully
    assert!(!processed_intent.original_text.is_empty());
    assert!(!prd.id.is_empty());
    assert!(!ard.id.is_empty());
    assert!(validation_result.valid);
    assert!(!risk_assessment.id.is_empty());
    assert!(planned_execution.is_ok());

    println!("E2E workflow completed successfully!");
}

#[tokio::test]
async fn test_llm_adapter_basic_functionality() {
    use venture_os_kernel::runtime::llm_adapter::{LLMAdapter, LLMConfig, LLMProvider};

    let config = LLMConfig {
        provider: LLMProvider::OpenAI,
        model: "gpt-3.5-turbo".to_string(),
        api_key: None,
        temperature: 0.7,
        base_url: None,
    };

    let llm_adapter = LLMAdapter::new(config);
    let response = llm_adapter.generate_text("Hello world").await;

    // Without API key, we expect network error (not mock success)
    assert!(response.is_err());
    let err = response.unwrap_err();
    println!("LLM adapter correctly fails without API key: {:?}", err);
}

#[test]
fn test_restate_adapter_basic_structure() {
    use venture_os_kernel::runtime::restate_adapter::{RestateAdapter, RestateConfig};

    let restate_config = RestateConfig {
        endpoint_url: "http://localhost:8080".to_string(),
        api_key: None,
        timeout_seconds: 30,
    };

    let _restate_adapter = RestateAdapter::new(restate_config);

    println!("Restate adapter structure test passed");
}

#[test]
fn test_xstate_adapter_basic_functionality() {
    use venture_os_kernel::runtime::xstate_adapter::{ExportFormat, XStateAdapter, XStateConfig};

    let xstate_config = XStateConfig {
        enable_visualization: true,
        export_format: ExportFormat::Json,
    };

    let xstate_adapter = XStateAdapter::new(xstate_config);

    // Create a simple plan to convert
    let plan = ExecutionPlanV1 {
        id: "test-plan-123".to_string(),
        version: "1.0.0".to_string(),
        parent_plan_id: None,
        intent_reference: "Test plan".to_string(),
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

    let xstate_result = xstate_adapter.to_xstate(&plan);
    assert!(xstate_result.is_ok());

    let xstate_machine = xstate_result.unwrap();
    assert_eq!(xstate_machine.id, "plan_test-plan-123");

    println!("XState adapter basic test passed");
}
