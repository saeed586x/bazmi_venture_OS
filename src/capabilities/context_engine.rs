//! Context Engine - manages environmental context and situational awareness

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context Engine - manages environmental context and situational awareness
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextEngine {
    /// Current environmental context
    context: EnvironmentContext,
    /// Historical context snapshots
    history: Vec<ContextSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentContext {
    /// Current time
    pub timestamp: DateTime<Utc>,
    /// Available resources
    pub resources: HashMap<String, ResourceInfo>,
    /// Environmental constraints
    pub constraints: Vec<EnvironmentalConstraint>,
    /// Stakeholder information
    pub stakeholders: HashMap<String, StakeholderInfo>,
    /// Market conditions
    pub market_conditions: MarketConditions,
    /// Technical environment
    pub technical_environment: TechnicalEnvironment,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    pub resource_type: String,
    pub availability: f64, // 0.0 to 1.0
    pub capacity: Option<f64>,
    pub cost: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentalConstraint {
    pub id: String,
    pub description: String,
    pub constraint_type: ConstraintType,
    pub severity: ConstraintSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Regulatory,
    Technical,
    Business,
    Operational,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakeholderInfo {
    pub name: String,
    pub role: String,
    pub influence: f64, // 0.0 to 1.0
    pub interests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    pub demand_trend: Trend,
    pub competition_level: CompetitionLevel,
    pub economic_indicators: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Trend {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompetitionLevel {
    Low,
    Moderate,
    High,
    Intense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalEnvironment {
    pub platforms: Vec<String>,
    pub technologies: Vec<String>,
    pub infrastructure_status: InfrastructureStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureStatus {
    pub uptime: f64, // 0.0 to 1.0
    pub performance: PerformanceMetrics,
    pub security_posture: SecurityPosture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub latency_ms: f64,
    pub throughput_ops: f64,
    pub error_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPosture {
    pub compliance_score: f64, // 0.0 to 1.0
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub severity: VulnerabilitySeverity,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnapshot {
    pub timestamp: DateTime<Utc>,
    pub context: EnvironmentContext,
}

impl ContextEngine {
    /// Create a new context engine
    pub fn new() -> Self {
        Self {
            context: EnvironmentContext {
                timestamp: Utc::now(),
                resources: HashMap::new(),
                constraints: vec![],
                stakeholders: HashMap::new(),
                market_conditions: MarketConditions {
                    demand_trend: Trend::Stable,
                    competition_level: CompetitionLevel::Moderate,
                    economic_indicators: HashMap::new(),
                },
                technical_environment: TechnicalEnvironment {
                    platforms: vec![],
                    technologies: vec![],
                    infrastructure_status: InfrastructureStatus {
                        uptime: 0.99,
                        performance: PerformanceMetrics {
                            latency_ms: 50.0,
                            throughput_ops: 1000.0,
                            error_rate: 0.001,
                        },
                        security_posture: SecurityPosture {
                            compliance_score: 0.95,
                            vulnerabilities: vec![],
                        },
                    },
                },
            },
            history: vec![],
        }
    }

    /// Update the current context
    pub fn update_context(&mut self, new_context: EnvironmentContext) {
        // Save current context to history
        let snapshot = ContextSnapshot {
            timestamp: self.context.timestamp,
            context: self.context.clone(),
        };
        self.history.push(snapshot);

        // Update current context
        self.context = new_context;
    }

    /// Get the current environment context
    pub fn get_current_context(&self) -> &EnvironmentContext {
        &self.context
    }

    /// Get context history
    pub fn get_history(&self) -> &[ContextSnapshot] {
        &self.history
    }

    /// Analyze context for decision making
    pub fn analyze_context(&self) -> ContextAnalysis {
        ContextAnalysis {
            resource_availability: self.calculate_resource_availability(),
            constraint_impact: self.assess_constraint_impact(),
            stakeholder_alignment: self.assess_stakeholder_alignment(),
            risk_factors: self.identify_risk_factors(),
        }
    }

    /// Calculate overall resource availability
    fn calculate_resource_availability(&self) -> f64 {
        if self.context.resources.is_empty() {
            return 1.0; // Default to fully available if no resources defined
        }

        let total_availability: f64 = self
            .context
            .resources
            .values()
            .map(|resource| resource.availability)
            .sum();

        total_availability / self.context.resources.len() as f64
    }

    /// Assess constraint impact
    fn assess_constraint_impact(&self) -> ConstraintImpact {
        let critical_constraints = self
            .context
            .constraints
            .iter()
            .filter(|c| matches!(c.severity, ConstraintSeverity::Critical))
            .count();

        let high_constraints = self
            .context
            .constraints
            .iter()
            .filter(|c| matches!(c.severity, ConstraintSeverity::High))
            .count();

        ConstraintImpact {
            critical_count: critical_constraints,
            high_count: high_constraints,
            overall_severity: if critical_constraints > 0 {
                ConstraintSeverity::Critical
            } else if high_constraints > 0 {
                ConstraintSeverity::High
            } else {
                ConstraintSeverity::Low
            },
        }
    }

    /// Assess stakeholder alignment
    fn assess_stakeholder_alignment(&self) -> f64 {
        if self.context.stakeholders.is_empty() {
            return 1.0; // Default to full alignment if no stakeholders defined
        }

        let total_influence: f64 = self
            .context
            .stakeholders
            .values()
            .map(|stakeholder| stakeholder.influence)
            .sum();

        total_influence / self.context.stakeholders.len() as f64
    }

    /// Identify risk factors from context
    fn identify_risk_factors(&self) -> Vec<RiskFactor> {
        let mut risks = Vec::new();

        // Check for critical infrastructure issues
        if self
            .context
            .technical_environment
            .infrastructure_status
            .uptime
            < 0.95
        {
            risks.push(RiskFactor {
                id: "infrastructure_uptime_low".to_string(),
                description: "Infrastructure uptime below acceptable threshold".to_string(),
                probability: 0.8,
                impact: 0.7,
                category: RiskCategory::Technical,
            });
        }

        // Check for security vulnerabilities
        let critical_vulns = self
            .context
            .technical_environment
            .infrastructure_status
            .security_posture
            .vulnerabilities
            .iter()
            .filter(|v| matches!(v.severity, VulnerabilitySeverity::Critical))
            .count();

        if critical_vulns > 0 {
            risks.push(RiskFactor {
                id: "critical_security_vulnerabilities".to_string(),
                description: format!(
                    "{} critical security vulnerabilities detected",
                    critical_vulns
                ),
                probability: 0.9,
                impact: 0.9,
                category: RiskCategory::Security,
            });
        }

        // Check for resource constraints
        let low_resources = self
            .context
            .resources
            .iter()
            .filter(|(_, resource)| resource.availability < 0.3)
            .count();

        if low_resources > 0 {
            risks.push(RiskFactor {
                id: "resource_constraints".to_string(),
                description: format!("{} resources with low availability", low_resources),
                probability: 0.6,
                impact: 0.5,
                category: RiskCategory::Operational,
            });
        }

        risks
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAnalysis {
    pub resource_availability: f64,
    pub constraint_impact: ConstraintImpact,
    pub stakeholder_alignment: f64,
    pub risk_factors: Vec<RiskFactor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintImpact {
    pub critical_count: usize,
    pub high_count: usize,
    pub overall_severity: ConstraintSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub id: String,
    pub description: String,
    pub probability: f64, // 0.0 to 1.0
    pub impact: f64,      // 0.0 to 1.0
    pub category: RiskCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskCategory {
    Technical,
    Security,
    Operational,
    Business,
    Market,
}

impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}
