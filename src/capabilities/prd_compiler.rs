//! PRD Compiler - compiles business intents into Product Requirements Documents

use crate::capabilities::requirements_engine::RequirementPriority;
use crate::contracts::ExecutionPlanV1;
use crate::core::semantic_model::SemanticModel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// PRD Compiler - compiles business intents into Product Requirements Documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDCompiler {
    /// Reference to the semantic model for domain understanding
    semantic_model: SemanticModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRequirementsDocument {
    pub id: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub product_overview: ProductOverview,
    pub business_requirements: Vec<BusinessRequirement>,
    pub functional_requirements: Vec<FunctionalRequirement>,
    pub non_functional_requirements: Vec<NonFunctionalRequirement>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    pub constraints: Vec<PRDConstraint>,
    pub assumptions: Vec<Assumption>,
    pub dependencies: Vec<Dependency>,
    pub success_metrics: Vec<SuccessMetric>,
    pub timeline: Timeline,
    pub stakeholders: Vec<Stakeholder>,
    pub approval_info: ApprovalInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductOverview {
    pub product_name: String,
    pub product_vision: String,
    pub problem_statement: String,
    pub solution_overview: String,
    pub target_audience: Vec<String>,
    pub key_features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: RequirementPriority,
    pub business_value: BusinessValue,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessValue {
    pub revenue_impact: Option<f64>,
    pub cost_savings: Option<f64>,
    pub efficiency_gain: Option<f64>,
    pub customer_satisfaction_impact: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: RequirementPriority,
    pub user_story: Option<UserStory>,
    pub acceptance_criteria: Vec<String>,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserStory {
    pub role: String,
    pub action: String,
    pub benefit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonFunctionalRequirement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub priority: RequirementPriority,
    pub requirement_type: NFRType,
    pub metrics: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NFRType {
    Performance,
    Security,
    Usability,
    Reliability,
    Scalability,
    Compatibility,
    Maintainability,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub requirement_id: String,
    pub description: String,
    pub test_method: String,
    pub pass_criteria: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDConstraint {
    pub id: String,
    pub description: String,
    pub constraint_type: ConstraintType,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Technical,
    Business,
    Regulatory,
    Resource,
    Time,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub id: String,
    pub description: String,
    pub validity_check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub id: String,
    pub description: String,
    pub dependent_on: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    Internal,
    External,
    Technical,
    Business,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessMetric {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_value: String,
    pub measurement_method: String,
    pub baseline: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: String,
    pub name: String,
    pub description: String,
    pub target_date: chrono::DateTime<chrono::Utc>,
    pub deliverables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stakeholder {
    pub id: String,
    pub name: String,
    pub role: String,
    pub responsibilities: Vec<String>,
    pub contact_info: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalInfo {
    pub approvers: Vec<Approver>,
    pub approval_status: ApprovalStatus,
    pub approval_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approver {
    pub id: String,
    pub name: String,
    pub role: String,
    pub approved: bool,
    pub approval_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Draft,
    Review,
    Approved,
    Rejected,
}

impl PRDCompiler {
    /// Create a new PRD compiler
    pub fn new(semantic_model: SemanticModel) -> Self {
        Self { semantic_model }
    }

    /// Compile an intent into a Product Requirements Document
    pub fn compile_from_intent(
        &self,
        intent: &str,
    ) -> Result<ProductRequirementsDocument, PRDError> {
        // In a real implementation, this would:
        // 1. Parse the intent using the semantic model
        // 2. Extract key concepts and entities
        // 3. Generate appropriate requirements
        // 4. Structure into a comprehensive PRD

        Ok(ProductRequirementsDocument {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("PRD for: {}", intent),
            version: "1.0.0".to_string(),
            description: intent.to_string(),
            product_overview: ProductOverview {
                product_name: self.extract_product_name(intent),
                product_vision: format!("A solution to {}", intent),
                problem_statement: format!("The problem of {}", intent),
                solution_overview: format!("A system that addresses {}", intent),
                target_audience: vec!["Primary users".to_string()],
                key_features: vec![
                    "Core functionality".to_string(),
                    "User interface".to_string(),
                    "Integration capabilities".to_string(),
                ],
            },
            business_requirements: vec![BusinessRequirement {
                id: "BR-001".to_string(),
                title: "Core Business Requirement".to_string(),
                description: format!("The system shall address the {}", intent),
                priority: RequirementPriority::High,
                business_value: BusinessValue {
                    revenue_impact: Some(100000.0),
                    cost_savings: Some(50000.0),
                    efficiency_gain: Some(0.2),
                    customer_satisfaction_impact: Some(0.15),
                },
                dependencies: vec![],
            }],
            functional_requirements: vec![FunctionalRequirement {
                id: "FR-001".to_string(),
                title: "Core Functional Requirement".to_string(),
                description: format!("The system shall provide functionality to {}", intent),
                priority: RequirementPriority::High,
                user_story: Some(UserStory {
                    role: "User".to_string(),
                    action: format!("perform {}", intent),
                    benefit: "to achieve the desired outcome".to_string(),
                }),
                acceptance_criteria: vec![
                    "Functionality is accessible".to_string(),
                    "Results are accurate".to_string(),
                ],
                dependencies: vec![],
            }],
            non_functional_requirements: vec![NonFunctionalRequirement {
                id: "NFR-001".to_string(),
                title: "Performance Requirement".to_string(),
                description: "System shall respond within 2 seconds".to_string(),
                priority: RequirementPriority::Medium,
                requirement_type: NFRType::Performance,
                metrics: [("response_time".to_string(), "2000ms".to_string())]
                    .iter()
                    .cloned()
                    .collect(),
            }],
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "AC-001".to_string(),
                requirement_id: "FR-001".to_string(),
                description: "Functionality performs as described".to_string(),
                test_method: "Manual testing".to_string(),
                pass_criteria: "All features work correctly".to_string(),
            }],
            constraints: vec![PRDConstraint {
                id: "C-001".to_string(),
                description: "Must comply with organizational standards".to_string(),
                constraint_type: ConstraintType::Business,
                impact: "Medium".to_string(),
            }],
            assumptions: vec![Assumption {
                id: "A-001".to_string(),
                description: "Users have basic computer skills".to_string(),
                validity_check: "User testing".to_string(),
            }],
            dependencies: vec![Dependency {
                id: "D-001".to_string(),
                description: "Requires database backend".to_string(),
                dependent_on: "Database Team".to_string(),
                dependency_type: DependencyType::Technical,
            }],
            success_metrics: vec![SuccessMetric {
                id: "SM-001".to_string(),
                name: "User Adoption Rate".to_string(),
                description: "Percentage of target users who adopt the system".to_string(),
                target_value: "70%".to_string(),
                measurement_method: "Analytics tracking".to_string(),
                baseline: None,
            }],
            timeline: Timeline {
                start_date: chrono::Utc::now(),
                end_date: chrono::Utc::now() + chrono::Duration::days(90),
                milestones: vec![Milestone {
                    id: "M-001".to_string(),
                    name: "Design Complete".to_string(),
                    description: "System design finalized".to_string(),
                    target_date: chrono::Utc::now() + chrono::Duration::days(30),
                    deliverables: vec!["System architecture document".to_string()],
                }],
            },
            stakeholders: vec![Stakeholder {
                id: "S-001".to_string(),
                name: "Product Owner".to_string(),
                role: "Owner".to_string(),
                responsibilities: vec!["Define requirements".to_string()],
                contact_info: "product.owner@company.com".to_string(),
            }],
            approval_info: ApprovalInfo {
                approvers: vec![Approver {
                    id: "AP-001".to_string(),
                    name: "Product Owner".to_string(),
                    role: "Owner".to_string(),
                    approved: false,
                    approval_date: None,
                }],
                approval_status: ApprovalStatus::Draft,
                approval_date: None,
            },
        })
    }

    /// Convert PRD to execution plan
    pub fn to_execution_plan(&self, prd: &ProductRequirementsDocument) -> ExecutionPlanV1 {
        ExecutionPlanV1 {
            id: uuid::Uuid::new_v4().to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: prd.title.clone(),
            goals: prd
                .business_requirements
                .iter()
                .map(|br| crate::contracts::execution_plan_v1::Goal {
                    id: br.id.clone(),
                    description: br.description.clone(),
                    priority: match br.priority {
                        RequirementPriority::Low => 1,
                        RequirementPriority::Medium => 2,
                        RequirementPriority::High => 3,
                        RequirementPriority::Critical => 4,
                    },
                })
                .collect(),
            constraints: prd
                .constraints
                .iter()
                .map(|c| crate::contracts::execution_plan_v1::Constraint {
                    id: c.id.clone(),
                    description: c.description.clone(),
                    constraint_type: match c.constraint_type {
                        ConstraintType::Technical => {
                            crate::contracts::execution_plan_v1::ConstraintType::Custom(
                                "Technical".to_string(),
                            )
                        }
                        ConstraintType::Business => {
                            crate::contracts::execution_plan_v1::ConstraintType::Custom(
                                "Business".to_string(),
                            )
                        }
                        ConstraintType::Regulatory => {
                            crate::contracts::execution_plan_v1::ConstraintType::Compliance
                        }
                        ConstraintType::Resource => {
                            crate::contracts::execution_plan_v1::ConstraintType::Resource
                        }
                        ConstraintType::Time => {
                            crate::contracts::execution_plan_v1::ConstraintType::Time
                        }
                        ConstraintType::Custom(ref s) => {
                            crate::contracts::execution_plan_v1::ConstraintType::Custom(s.clone())
                        }
                    },
                })
                .collect(),
            required_capabilities: vec!["development".to_string(), "testing".to_string()],
            inputs: vec![],
            tasks: prd
                .functional_requirements
                .iter()
                .map(|fr| crate::contracts::execution_plan_v1::Task {
                    id: fr.id.clone(),
                    name: fr.title.clone(),
                    description: fr.description.clone(),
                    capability: "development".to_string(),
                    parameters: std::collections::HashMap::new(),
                    expected_duration: Some(10 * 24 * 60 * 60), // 10 days in seconds
                })
                .collect(),
            dependencies: vec![],
            artifacts: vec![],
            gates: vec![],
            completion_conditions: vec![],
            retry_policy: None,
            provenance: None,
            creation_timestamp: chrono::Utc::now(),
            replan_reason: None,
        }
    }

    /// Extract product name from intent
    fn extract_product_name(&self, intent: &str) -> String {
        // Simple extraction - in reality this would be more sophisticated
        if intent.len() > 20 {
            format!("{}...", &intent[..20])
        } else {
            intent.to_string()
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum PRDError {
    #[error("Invalid intent: {0}")]
    InvalidIntent(String),
    #[error("Compilation failed: {0}")]
    CompilationFailed(String),
}
