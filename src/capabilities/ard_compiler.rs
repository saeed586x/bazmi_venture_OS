//! ARD Compiler - compiles requirements into Architecture Requirements Documents

use crate::capabilities::domain_engine::DomainModel;
use crate::core::semantic_model::SemanticModel;
use serde::{Deserialize, Serialize};

/// ARD Compiler - compiles requirements into Architecture Requirements Documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARDCompiler {
    /// Reference to the semantic model for domain understanding
    semantic_model: SemanticModel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureRequirementsDocument {
    pub id: String,
    pub title: String,
    pub version: String,
    pub description: String,
    pub system_overview: SystemOverview,
    pub architectural_constraints: Vec<ArchitecturalConstraint>,
    pub quality_attributes: Vec<QualityAttribute>,
    pub functional_components: Vec<FunctionalComponent>,
    pub data_architecture: DataArchitecture,
    pub integration_patterns: Vec<IntegrationPattern>,
    pub deployment_topology: DeploymentTopology,
    pub security_requirements: Vec<SecurityRequirement>,
    pub operational_requirements: Vec<OperationalRequirement>,
    pub technology_stack: TechnologyStack,
    pub scalability_requirements: Vec<ScalabilityRequirement>,
    pub maintainability_requirements: Vec<MaintainabilityRequirement>,
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    pub system_name: String,
    pub system_purpose: String,
    pub system_scope: String,
    pub stakeholders: Vec<SystemStakeholder>,
    pub context_diagram: Option<String>, // URL or reference to diagram
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStakeholder {
    pub id: String,
    pub name: String,
    pub role: String,
    pub concerns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalConstraint {
    pub id: String,
    pub description: String,
    pub constraint_type: ConstraintType,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Technical,
    Business,
    Regulatory,
    Platform,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAttribute {
    pub id: String,
    pub name: String,
    pub description: String,
    pub priority: Priority,
    pub target_value: String,
    pub measurement_approach: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionalComponent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub responsibilities: Vec<String>,
    pub interfaces: Vec<ComponentInterface>,
    pub dependencies: Vec<String>,
    pub technologies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInterface {
    pub id: String,
    pub name: String,
    pub protocol: String,
    pub data_format: String,
    pub security_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataArchitecture {
    pub data_entities: Vec<DataEntity>,
    pub data_flow_patterns: Vec<DataFlowPattern>,
    pub storage_requirements: Vec<StorageRequirement>,
    pub data_governance: DataGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataEntity {
    pub id: String,
    pub name: String,
    pub description: String,
    pub attributes: Vec<DataAttribute>,
    pub relationships: Vec<DataRelationship>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAttribute {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRelationship {
    pub id: String,
    pub source_entity: String,
    pub target_entity: String,
    pub relationship_type: RelationshipType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pattern_type: DataFlowType,
    pub components_involved: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataFlowType {
    RequestResponse,
    EventDriven,
    Batch,
    Stream,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageRequirement {
    pub id: String,
    pub data_type: String,
    pub volume_estimate: String,
    pub retention_period: String,
    pub backup_requirements: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataGovernance {
    pub data_ownership: Vec<DataOwner>,
    pub privacy_requirements: Vec<String>,
    pub audit_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataOwner {
    pub entity_id: String,
    pub owner_name: String,
    pub owner_role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationPattern {
    pub id: String,
    pub name: String,
    pub description: String,
    pub pattern_type: IntegrationType,
    pub protocols_used: Vec<String>,
    pub security_considerations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegrationType {
    API,
    MessageQueue,
    Database,
    FileTransfer,
    EventStreaming,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentTopology {
    pub environments: Vec<Environment>,
    pub deployment_units: Vec<DeploymentUnit>,
    pub network_topology: NetworkTopology,
    pub failover_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub configuration: EnvironmentConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfiguration {
    pub infrastructure: String,
    pub security_level: String,
    pub monitoring_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentUnit {
    pub id: String,
    pub name: String,
    pub components: Vec<String>,
    pub resource_requirements: ResourceRequirements,
    pub scaling_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: f64,
    pub memory_gb: f64,
    pub storage_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub zones: Vec<NetworkZone>,
    pub connectivity_rules: Vec<ConnectivityRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkZone {
    pub id: String,
    pub name: String,
    pub security_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityRule {
    pub id: String,
    pub source_zone: String,
    pub target_zone: String,
    pub allowed_protocols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirement {
    pub id: String,
    pub category: SecurityCategory,
    pub description: String,
    pub implementation_guidance: String,
    pub compliance_mapping: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityCategory {
    Authentication,
    Authorization,
    Encryption,
    Audit,
    DataProtection,
    NetworkSecurity,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationalRequirement {
    pub id: String,
    pub category: OperationalCategory,
    pub description: String,
    pub service_level: String,
    pub monitoring_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationalCategory {
    Availability,
    Performance,
    Reliability,
    Maintainability,
    Monitoring,
    Backup,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyStack {
    pub programming_languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub middleware: Vec<String>,
    pub infrastructure: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalabilityRequirement {
    pub id: String,
    pub metric: String,
    pub target_value: String,
    pub scaling_strategy: String,
    pub bottlenecks_identified: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintainabilityRequirement {
    pub id: String,
    pub aspect: MaintainabilityAspect,
    pub requirement: String,
    pub tools_supported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintainabilityAspect {
    CodeStructure,
    Documentation,
    Testing,
    Deployment,
    Monitoring,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub id: String,
    pub standard: String,
    pub description: String,
    pub validation_method: String,
    pub evidence_required: Vec<String>,
}

impl ARDCompiler {
    /// Create a new ARD compiler
    pub fn new(semantic_model: SemanticModel) -> Self {
        Self { semantic_model }
    }

    /// Compile domain model into Architecture Requirements Document
    pub fn compile_from_domain_model(
        &self,
        domain_model: &DomainModel,
    ) -> ArchitectureRequirementsDocument {
        ArchitectureRequirementsDocument {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("ARD for {}", domain_model.name),
            version: domain_model.version.clone(),
            description: domain_model.description.clone(),
            system_overview: SystemOverview {
                system_name: domain_model.name.clone(),
                system_purpose: format!("System to manage {}", domain_model.name),
                system_scope: "Enterprise-wide".to_string(),
                stakeholders: vec![SystemStakeholder {
                    id: "stakeholder-1".to_string(),
                    name: "System Architect".to_string(),
                    role: "Designer".to_string(),
                    concerns: vec!["Scalability".to_string(), "Maintainability".to_string()],
                }],
                context_diagram: None,
            },
            architectural_constraints: vec![ArchitecturalConstraint {
                id: "constraint-1".to_string(),
                description: "Must follow enterprise architecture standards".to_string(),
                constraint_type: ConstraintType::Business,
                rationale: "Ensures consistency across systems".to_string(),
            }],
            quality_attributes: vec![QualityAttribute {
                id: "qa-1".to_string(),
                name: "Availability".to_string(),
                description: "System uptime requirement".to_string(),
                priority: Priority::High,
                target_value: "99.9%".to_string(),
                measurement_approach: "Monitoring tools".to_string(),
            }],
            functional_components: domain_model
                .entities
                .iter()
                .map(|entity| FunctionalComponent {
                    id: format!("component-{}", entity.id),
                    name: entity.name.clone(),
                    description: entity.description.clone(),
                    responsibilities: vec![format!("Manage {} entities", entity.name)],
                    interfaces: vec![],
                    dependencies: vec![],
                    technologies: vec!["Rust".to_string(), "PostgreSQL".to_string()],
                })
                .collect(),
            data_architecture: self.generate_data_architecture(domain_model),
            integration_patterns: vec![IntegrationPattern {
                id: "integration-1".to_string(),
                name: "REST API".to_string(),
                description: "Primary interface for external systems".to_string(),
                pattern_type: IntegrationType::API,
                protocols_used: vec!["HTTP/HTTPS".to_string()],
                security_considerations: vec!["OAuth 2.0 authentication".to_string()],
            }],
            deployment_topology: DeploymentTopology {
                environments: vec![Environment {
                    id: "env-dev".to_string(),
                    name: "Development".to_string(),
                    purpose: "Development and testing".to_string(),
                    configuration: EnvironmentConfiguration {
                        infrastructure: "Containerized".to_string(),
                        security_level: "Standard".to_string(),
                        monitoring_enabled: true,
                    },
                }],
                deployment_units: vec![],
                network_topology: NetworkTopology {
                    zones: vec![NetworkZone {
                        id: "zone-public".to_string(),
                        name: "Public Zone".to_string(),
                        security_level: "Standard".to_string(),
                    }],
                    connectivity_rules: vec![],
                },
                failover_strategy: "Automatic failover with load balancing".to_string(),
            },
            security_requirements: vec![SecurityRequirement {
                id: "sec-1".to_string(),
                category: SecurityCategory::Authentication,
                description: "All API endpoints require authentication".to_string(),
                implementation_guidance: "Implement OAuth 2.0 JWT tokens".to_string(),
                compliance_mapping: vec!["ISO 27001".to_string()],
            }],
            operational_requirements: vec![OperationalRequirement {
                id: "op-1".to_string(),
                category: OperationalCategory::Availability,
                description: "System must be available 99.9% of the time".to_string(),
                service_level: "99.9% uptime".to_string(),
                monitoring_requirements: vec!["Health checks every 5 minutes".to_string()],
            }],
            technology_stack: TechnologyStack {
                programming_languages: vec!["Rust".to_string()],
                frameworks: vec!["Axum".to_string(), "Tokio".to_string()],
                databases: vec!["PostgreSQL".to_string()],
                middleware: vec!["Redis".to_string()],
                infrastructure: vec!["Docker".to_string(), "Kubernetes".to_string()],
            },
            scalability_requirements: vec![ScalabilityRequirement {
                id: "scale-1".to_string(),
                metric: "Requests per second".to_string(),
                target_value: "10,000 RPS".to_string(),
                scaling_strategy: "Horizontal pod scaling".to_string(),
                bottlenecks_identified: vec!["Database connections".to_string()],
            }],
            maintainability_requirements: vec![MaintainabilityRequirement {
                id: "maint-1".to_string(),
                aspect: MaintainabilityAspect::CodeStructure,
                requirement: "Follow Rust coding standards and use clippy".to_string(),
                tools_supported: vec!["rustfmt".to_string(), "clippy".to_string()],
            }],
            compliance_requirements: vec![ComplianceRequirement {
                id: "comp-1".to_string(),
                standard: "GDPR".to_string(),
                description: "Personal data protection requirements".to_string(),
                validation_method: "Regular audits".to_string(),
                evidence_required: vec!["Data processing records".to_string()],
            }],
        }
    }

    /// Generate data architecture from domain model
    fn generate_data_architecture(&self, domain_model: &DomainModel) -> DataArchitecture {
        DataArchitecture {
            data_entities: domain_model
                .entities
                .iter()
                .map(|entity| {
                    DataEntity {
                        id: entity.id.clone(),
                        name: entity.name.clone(),
                        description: entity.description.clone(),
                        attributes: entity
                            .attributes
                            .iter()
                            .map(|attr| DataAttribute {
                                name: attr.name.clone(),
                                data_type: attr.data_type.clone(),
                                required: attr.required,
                                constraints: attr.constraints.clone(),
                            })
                            .collect(),
                        relationships: vec![], // Would be populated based on domain relationships
                    }
                })
                .collect(),
            data_flow_patterns: vec![DataFlowPattern {
                id: "flow-1".to_string(),
                name: "Request-Response".to_string(),
                description: "Synchronous API requests".to_string(),
                pattern_type: DataFlowType::RequestResponse,
                components_involved: domain_model
                    .entities
                    .iter()
                    .map(|e| format!("component-{}", e.id))
                    .collect(),
            }],
            storage_requirements: vec![StorageRequirement {
                id: "storage-1".to_string(),
                data_type: "Structured".to_string(),
                volume_estimate: "100GB".to_string(),
                retention_period: "5 years".to_string(),
                backup_requirements: "Daily backups with 30-day retention".to_string(),
            }],
            data_governance: DataGovernance {
                data_ownership: vec![],
                privacy_requirements: vec!["Data encryption at rest".to_string()],
                audit_requirements: vec!["All data changes logged".to_string()],
            },
        }
    }
}
