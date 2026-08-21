//! Risk Engine - identifies, assesses, and mitigates risks

use crate::capabilities::context_engine::ContextEngine;
use crate::contracts::ExecutionPlanV1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk Engine - identifies, assesses, and mitigates risks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEngine {
    /// Risk categories and their assessment criteria
    risk_categories: HashMap<String, RiskCategoryDefinition>,
    /// Risk mitigation strategies
    mitigation_strategies: HashMap<String, MitigationStrategy>,
    /// Reference to context engine for environmental risks
    context_engine: ContextEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskCategoryDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub assessment_criteria: Vec<AssessmentCriterion>,
    pub typical_impact_range: ImpactRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentCriterion {
    pub id: String,
    pub name: String,
    pub description: String,
    pub weight: f64, // 0.0 to 1.0
    pub measurement_method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactRange {
    pub min_impact: f64,
    pub max_impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub strategy_type: StrategyType,
    pub implementation_cost: CostEstimate,
    pub effectiveness_rating: f64, // 0.0 to 1.0
    pub implementation_time_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyType {
    Avoidance,
    Mitigation,
    Transfer,
    Acceptance,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub min_cost: f64,
    pub max_cost: f64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: String,
    pub assessed_item: String,
    pub assessed_item_type: AssessedItemType,
    pub risks: Vec<Risk>,
    pub overall_risk_score: f64, // 0.0 to 1.0
    pub risk_level: RiskLevel,
    pub assessment_date: chrono::DateTime<chrono::Utc>,
    pub next_review_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssessedItemType {
    ExecutionPlan,
    Requirement,
    Component,
    Project,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub probability: f64, // 0.0 to 1.0
    pub impact: f64,      // 0.0 to 1.0
    pub risk_score: f64,  // probability * impact
    pub risk_level: RiskLevel,
    pub owner: String,
    pub mitigation_plan: Option<MitigationPlan>,
    pub triggers: Vec<RiskTrigger>,
    pub monitoring_indicators: Vec<MonitoringIndicator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationPlan {
    pub id: String,
    pub description: String,
    pub strategy_id: String,
    pub actions: Vec<MitigationAction>,
    pub responsible_party: String,
    pub target_completion_date: chrono::DateTime<chrono::Utc>,
    pub status: MitigationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationAction {
    pub id: String,
    pub description: String,
    pub assigned_to: String,
    pub due_date: chrono::DateTime<chrono::Utc>,
    pub completed_date: Option<chrono::DateTime<chrono::Utc>>,
    pub status: ActionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitigationStatus {
    Planned,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    NotStarted,
    InProgress,
    Completed,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTrigger {
    pub id: String,
    pub description: String,
    pub indicator_type: TriggerType,
    pub threshold_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerType {
    MetricThreshold,
    EventBased,
    TimeBased,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringIndicator {
    pub id: String,
    pub name: String,
    pub description: String,
    pub measurement_frequency: String,
    pub responsible_party: String,
}

impl RiskEngine {
    /// Create a new risk engine
    pub fn new(context_engine: ContextEngine) -> Self {
        let mut risk_categories = HashMap::new();

        // Define standard risk categories
        risk_categories.insert(
            "technical".to_string(),
            RiskCategoryDefinition {
                id: "technical".to_string(),
                name: "Technical Risk".to_string(),
                description: "Risks related to technology, architecture, and implementation"
                    .to_string(),
                assessment_criteria: vec![
                    AssessmentCriterion {
                        id: "tech-complexity".to_string(),
                        name: "Technical Complexity".to_string(),
                        description: "Complexity of the technical solution".to_string(),
                        weight: 0.3,
                        measurement_method: "Expert assessment".to_string(),
                    },
                    AssessmentCriterion {
                        id: "tech-maturity".to_string(),
                        name: "Technology Maturity".to_string(),
                        description: "Maturity level of selected technologies".to_string(),
                        weight: 0.25,
                        measurement_method: "Technology radar assessment".to_string(),
                    },
                ],
                typical_impact_range: ImpactRange {
                    min_impact: 0.1,
                    max_impact: 0.9,
                },
            },
        );

        risk_categories.insert(
            "schedule".to_string(),
            RiskCategoryDefinition {
                id: "schedule".to_string(),
                name: "Schedule Risk".to_string(),
                description: "Risks related to project timeline and delivery dates".to_string(),
                assessment_criteria: vec![
                    AssessmentCriterion {
                        id: "sched-complexity".to_string(),
                        name: "Schedule Complexity".to_string(),
                        description: "Complexity of the project schedule".to_string(),
                        weight: 0.4,
                        measurement_method: "Critical path analysis".to_string(),
                    },
                    AssessmentCriterion {
                        id: "sched-buffer".to_string(),
                        name: "Schedule Buffer".to_string(),
                        description: "Amount of schedule buffer available".to_string(),
                        weight: 0.35,
                        measurement_method: "Buffer percentage calculation".to_string(),
                    },
                ],
                typical_impact_range: ImpactRange {
                    min_impact: 0.2,
                    max_impact: 0.8,
                },
            },
        );

        let mut mitigation_strategies = HashMap::new();

        // Define standard mitigation strategies
        mitigation_strategies.insert(
            "prototyping".to_string(),
            MitigationStrategy {
                id: "prototyping".to_string(),
                name: "Prototyping".to_string(),
                description: "Build prototypes to validate technical approaches".to_string(),
                strategy_type: StrategyType::Mitigation,
                implementation_cost: CostEstimate {
                    min_cost: 1000.0,
                    max_cost: 5000.0,
                    currency: "USD".to_string(),
                },
                effectiveness_rating: 0.8,
                implementation_time_days: 14,
            },
        );

        Self {
            risk_categories,
            mitigation_strategies,
            context_engine,
        }
    }

    /// Register a new risk category
    pub fn register_risk_category(&mut self, category: RiskCategoryDefinition) {
        self.risk_categories.insert(category.id.clone(), category);
    }

    /// Register a new mitigation strategy
    pub fn register_mitigation_strategy(&mut self, strategy: MitigationStrategy) {
        self.mitigation_strategies
            .insert(strategy.id.clone(), strategy);
    }

    /// Assess risks for an execution plan
    pub fn assess_execution_plan_risks(&self, plan: &ExecutionPlanV1) -> RiskAssessment {
        let mut risks = Vec::new();

        // Assess technical risks based on plan complexity
        let task_count = plan.tasks.len();
        if task_count > 20 {
            risks.push(Risk {
                id: "tech-complex-high".to_string(),
                name: "High Technical Complexity".to_string(),
                description: format!(
                    "Plan contains {} tasks, indicating high complexity",
                    task_count
                ),
                category: "technical".to_string(),
                probability: 0.7,
                impact: 0.6,
                risk_score: 0.7 * 0.6,
                risk_level: RiskLevel::High,
                owner: "Technical Lead".to_string(),
                mitigation_plan: Some(self.create_mitigation_plan(
                    "prototyping",
                    "Create prototypes for complex components",
                    "Tech Lead",
                )),
                triggers: vec![RiskTrigger {
                    id: "complexity-trigger".to_string(),
                    description: "Task count exceeds complexity threshold".to_string(),
                    indicator_type: TriggerType::MetricThreshold,
                    threshold_value: "20 tasks".to_string(),
                }],
                monitoring_indicators: vec![MonitoringIndicator {
                    id: "tech-debt-indicator".to_string(),
                    name: "Technical Debt".to_string(),
                    description: "Measure of accumulated technical debt".to_string(),
                    measurement_frequency: "Weekly".to_string(),
                    responsible_party: "Development Team".to_string(),
                }],
            });
        }

        // Assess schedule risks based on plan duration
        let total_duration: u64 = plan.tasks.iter().filter_map(|t| t.expected_duration).sum();

        if total_duration > 7776000 {
            // 90 days in seconds
            risks.push(Risk {
                id: "sched-overrun".to_string(),
                name: "Schedule Overrun Risk".to_string(),
                description: "Plan duration exceeds 90 days, increasing overrun risk".to_string(),
                category: "schedule".to_string(),
                probability: 0.6,
                impact: 0.7,
                risk_score: 0.6 * 0.7,
                risk_level: RiskLevel::High,
                owner: "Project Manager".to_string(),
                mitigation_plan: None,
                triggers: vec![],
                monitoring_indicators: vec![],
            });
        }

        // Assess resource risks based on required capabilities
        let unique_capabilities = plan.required_capabilities.len();
        if unique_capabilities > 5 {
            risks.push(Risk {
                id: "res-scarcity".to_string(),
                name: "Resource Scarcity Risk".to_string(),
                description: format!(
                    "Plan requires {} different capabilities, may strain resources",
                    unique_capabilities
                ),
                category: "technical".to_string(),
                probability: 0.5,
                impact: 0.5,
                risk_score: 0.5 * 0.5,
                risk_level: RiskLevel::Medium,
                owner: "Resource Manager".to_string(),
                mitigation_plan: None,
                triggers: vec![],
                monitoring_indicators: vec![],
            });
        }

        // Assess context-based risks
        let context_analysis = self.context_engine.analyze_context();
        for risk_factor in context_analysis.risk_factors {
            risks.push(Risk {
                id: format!("context-{}", risk_factor.id),
                name: risk_factor.description.clone(),
                description: format!("Environmental risk factor: {}", risk_factor.description),
                category: match risk_factor.category {
                    crate::capabilities::context_engine::RiskCategory::Technical => {
                        "technical".to_string()
                    }
                    crate::capabilities::context_engine::RiskCategory::Security => {
                        "security".to_string()
                    }
                    crate::capabilities::context_engine::RiskCategory::Operational => {
                        "operational".to_string()
                    }
                    crate::capabilities::context_engine::RiskCategory::Business => {
                        "business".to_string()
                    }
                    crate::capabilities::context_engine::RiskCategory::Market => {
                        "market".to_string()
                    }
                },
                probability: risk_factor.probability,
                impact: risk_factor.impact,
                risk_score: risk_factor.probability * risk_factor.impact,
                risk_level: self.determine_risk_level(risk_factor.probability * risk_factor.impact),
                owner: "Risk Manager".to_string(),
                mitigation_plan: None,
                triggers: vec![],
                monitoring_indicators: vec![],
            });
        }

        // Calculate overall risk score
        let overall_risk_score = if !risks.is_empty() {
            risks.iter().map(|r| r.risk_score).sum::<f64>() / risks.len() as f64
        } else {
            0.0
        };

        let overall_risk_level = self.determine_risk_level(overall_risk_score);

        RiskAssessment {
            id: format!("ra-{}", uuid::Uuid::new_v4()),
            assessed_item: plan.id.clone(),
            assessed_item_type: AssessedItemType::ExecutionPlan,
            risks,
            overall_risk_score,
            risk_level: overall_risk_level,
            assessment_date: chrono::Utc::now(),
            next_review_date: chrono::Utc::now() + chrono::Duration::days(30),
        }
    }

    /// Create a mitigation plan
    fn create_mitigation_plan(
        &self,
        strategy_id: &str,
        description: &str,
        responsible: &str,
    ) -> MitigationPlan {
        MitigationPlan {
            id: format!("mp-{}", uuid::Uuid::new_v4()),
            description: description.to_string(),
            strategy_id: strategy_id.to_string(),
            actions: vec![MitigationAction {
                id: format!("ma-{}", uuid::Uuid::new_v4()),
                description: format!("Execute {} strategy", strategy_id),
                assigned_to: responsible.to_string(),
                due_date: chrono::Utc::now() + chrono::Duration::days(14),
                completed_date: None,
                status: ActionStatus::NotStarted,
            }],
            responsible_party: responsible.to_string(),
            target_completion_date: chrono::Utc::now() + chrono::Duration::days(30),
            status: MitigationStatus::Planned,
        }
    }

    /// Determine risk level based on risk score
    fn determine_risk_level(&self, risk_score: f64) -> RiskLevel {
        match risk_score {
            s if s >= 0.7 => RiskLevel::Critical,
            s if s >= 0.5 => RiskLevel::High,
            s if s >= 0.3 => RiskLevel::Medium,
            _ => RiskLevel::Low,
        }
    }

    /// Get risk categories
    pub fn risk_categories(&self) -> &HashMap<String, RiskCategoryDefinition> {
        &self.risk_categories
    }

    /// Get mitigation strategies
    pub fn mitigation_strategies(&self) -> &HashMap<String, MitigationStrategy> {
        &self.mitigation_strategies
    }
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new(ContextEngine::new())
    }
}
