//! Planning Engine - creates and optimizes execution plans

use crate::capabilities::context_engine::ContextEngine;
use crate::contracts::ExecutionPlanV1;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Planning Engine - creates and optimizes execution plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningEngine {
    /// Planning algorithms
    algorithms: HashMap<String, PlanningAlgorithm>,
    /// Optimization strategies
    optimization_strategies: HashMap<String, OptimizationStrategy>,
    /// Reference to context engine for environmental awareness
    context_engine: ContextEngine,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningAlgorithm {
    pub id: String,
    pub name: String,
    pub description: String,
    pub algorithm_type: AlgorithmType,
    pub complexity: ComplexityLevel,
    pubapplicable_scenarios: String, // Applicable scenarios
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlgorithmType {
    TopologicalSort,
    CriticalPath,
    GeneticAlgorithm,
    Greedy,
    DynamicProgramming,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationStrategy {
    pub id: String,
    pub name: String,
    pub description: String,
    pub strategy_type: OptimizationType,
    pub objective: OptimizationObjective,
    pub constraints: Vec<OptimizationConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationType {
    ResourceAllocation,
    ScheduleOptimization,
    CostMinimization,
    RiskMitigation,
    PerformanceMaximization,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationObjective {
    MinimizeCost,
    MaximizeThroughput,
    MinimizeDuration,
    MaximizeResourceUtilization,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConstraint {
    pub id: String,
    pub description: String,
    pub constraint_type: ConstraintType,
    pub expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    Hard, // Must be satisfied
    Soft, // Preferred but not required
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningContext {
    pub goals: Vec<crate::contracts::execution_plan_v1::Goal>,
    pub constraints: Vec<crate::contracts::execution_plan_v1::Constraint>,
    pub available_capabilities: Vec<String>,
    pub resource_limits: HashMap<String, f64>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub priority: u32, // 1-10 scale
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedExecution {
    pub plan: ExecutionPlanV1,
    pub planning_metadata: PlanningMetadata,
    pub optimization_results: Vec<OptimizationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanningMetadata {
    pub algorithm_used: String,
    pub planning_duration_ms: u64,
    pub task_count: usize,
    pub dependency_count: usize,
    pub critical_path_length: u64,                  // in seconds
    pub resource_utilization: HashMap<String, f64>, // resource -> utilization %
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub strategy_id: String,
    pub objective: OptimizationObjective,
    pub improvement: f64, // percentage improvement
    pub metrics_before: HashMap<String, f64>,
    metrics_after: HashMap<String, f64>,
    applied: bool,
}

impl PlanningEngine {
    /// Create a new planning engine
    pub fn new(context_engine: ContextEngine) -> Self {
        let mut algorithms = HashMap::new();

        // Register standard planning algorithms
        algorithms.insert(
            "topological-sort".to_string(),
            PlanningAlgorithm {
                id: "topological-sort".to_string(),
                name: "Topological Sort".to_string(),
                description: "Orders tasks based on dependencies".to_string(),
                algorithm_type: AlgorithmType::TopologicalSort,
                complexity: ComplexityLevel::Low,
                pubapplicable_scenarios: "Tasks with clear dependencies".to_string(),
            },
        );

        algorithms.insert(
            "critical-path".to_string(),
            PlanningAlgorithm {
                id: "critical-path".to_string(),
                name: "Critical Path Method".to_string(),
                description: "Identifies longest sequence of dependent tasks".to_string(),
                algorithm_type: AlgorithmType::CriticalPath,
                complexity: ComplexityLevel::Medium,
                pubapplicable_scenarios: "Projects with interdependent tasks".to_string(),
            },
        );

        let mut optimization_strategies = HashMap::new();

        // Register standard optimization strategies
        optimization_strategies.insert(
            "resource-leveling".to_string(),
            OptimizationStrategy {
                id: "resource-leveling".to_string(),
                name: "Resource Leveling".to_string(),
                description: "Optimize resource allocation to avoid overallocation".to_string(),
                strategy_type: OptimizationType::ResourceAllocation,
                objective: OptimizationObjective::MaximizeResourceUtilization,
                constraints: vec![OptimizationConstraint {
                    id: "resource-limit".to_string(),
                    description: "Respect resource limits".to_string(),
                    constraint_type: ConstraintType::Hard,
                    expression: "resource_usage <= resource_limit".to_string(),
                }],
            },
        );

        Self {
            algorithms,
            optimization_strategies,
            context_engine,
        }
    }

    /// Register a planning algorithm
    pub fn register_algorithm(&mut self, algorithm: PlanningAlgorithm) {
        self.algorithms.insert(algorithm.id.clone(), algorithm);
    }

    /// Register an optimization strategy
    pub fn register_optimization_strategy(&mut self, strategy: OptimizationStrategy) {
        self.optimization_strategies
            .insert(strategy.id.clone(), strategy);
    }

    /// Create an execution plan from planning context
    pub fn create_plan(&self, context: PlanningContext) -> Result<PlannedExecution, PlanningError> {
        let start_time = std::time::Instant::now();

        // Create initial plan structure
        let mut plan = ExecutionPlanV1 {
            id: uuid::Uuid::new_v4().to_string(),
            version: "1.0.0".to_string(),
            parent_plan_id: None,
            intent_reference: "Generated plan".to_string(),
            goals: context.goals.clone(),
            constraints: context.constraints.clone(),
            required_capabilities: context.available_capabilities.clone(),
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

        // Generate tasks based on goals
        self.generate_tasks(&mut plan, &context);

        // Create dependencies
        self.create_dependencies(&mut plan);

        // Optimize the plan
        let optimization_results = self.optimize_plan(&mut plan, &context);

        // Calculate planning metadata
        let planning_duration = start_time.elapsed().as_millis() as u64;
        let planning_metadata = self.calculate_planning_metadata(&plan, planning_duration);

        Ok(PlannedExecution {
            plan,
            planning_metadata,
            optimization_results,
        })
    }

    /// Generate tasks based on goals
    fn generate_tasks(&self, plan: &mut ExecutionPlanV1, context: &PlanningContext) {
        for (index, goal) in context.goals.iter().enumerate() {
            // Create a task for each goal
            plan.tasks.push(crate::contracts::execution_plan_v1::Task {
                id: format!("task-{}", index + 1),
                name: format!("Achieve goal: {}", goal.description),
                description: goal.description.clone(),
                capability: if context.available_capabilities.is_empty() {
                    "general".to_string()
                } else {
                    context.available_capabilities[0].clone()
                },
                parameters: HashMap::new(),
                expected_duration: Some(86400), // 1 day default
            });
        }

        // Add context-aware tasks based on environmental analysis
        let context_analysis = self.context_engine.analyze_context();
        if context_analysis.resource_availability < 0.5 {
            plan.tasks.push(crate::contracts::execution_plan_v1::Task {
                id: "resource-assessment".to_string(),
                name: "Resource Assessment".to_string(),
                description: "Assess available resources and constraints".to_string(),
                capability: "analysis".to_string(),
                parameters: HashMap::new(),
                expected_duration: Some(3600), // 1 hour
            });
        }
    }

    /// Create dependencies between tasks
    fn create_dependencies(&self, plan: &mut ExecutionPlanV1) {
        // Simple dependency creation: each task depends on the previous one
        for i in 1..plan.tasks.len() {
            plan.dependencies
                .push(crate::contracts::execution_plan_v1::Dependency {
                    dependent_task_id: plan.tasks[i].id.clone(),
                    dependency_task_id: plan.tasks[i - 1].id.clone(),
                });
        }
    }

    /// Optimize the execution plan
    fn optimize_plan(
        &self,
        plan: &mut ExecutionPlanV1,
        context: &PlanningContext,
    ) -> Vec<OptimizationResult> {
        let mut results = Vec::new();

        // Apply each optimization strategy
        for strategy in self.optimization_strategies.values() {
            let result = self.apply_optimization_strategy(plan, context, strategy);
            results.push(result);
        }

        results
    }

    /// Apply an optimization strategy to the plan
    fn apply_optimization_strategy(
        &self,
        plan: &mut ExecutionPlanV1,
        context: &PlanningContext,
        strategy: &OptimizationStrategy,
    ) -> OptimizationResult {
        let metrics_before = self.collect_plan_metrics(plan);

        // Apply the optimization based on strategy type
        match strategy.strategy_type {
            OptimizationType::ResourceAllocation => {
                self.optimize_resource_allocation(plan, context);
            }
            OptimizationType::ScheduleOptimization => {
                self.optimize_schedule(plan);
            }
            _ => {} // Other optimizations not implemented yet
        }

        let metrics_after = self.collect_plan_metrics(plan);

        // Calculate improvement
        let improvement = if let (Some(&before_duration), Some(&after_duration)) = (
            metrics_before.get("total_duration"),
            metrics_after.get("total_duration"),
        ) {
            if before_duration > 0.0 {
                ((before_duration - after_duration) / before_duration) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        OptimizationResult {
            strategy_id: strategy.id.clone(),
            objective: strategy.objective.clone(),
            improvement,
            metrics_before,
            metrics_after,
            applied: true,
        }
    }

    /// Optimize resource allocation
    fn optimize_resource_allocation(&self, plan: &mut ExecutionPlanV1, _context: &PlanningContext) {
        // In a real implementation, this would optimize resource allocation
        // For now, we'll just ensure resource limits are respected

        // Check if any task exceeds resource limits
        for task in &mut plan.tasks {
            // This is a simplified check - in reality would be more complex
            if task.expected_duration.unwrap_or(0) > 604800 {
                // 7 days
                // Reduce duration estimate for very long tasks
                task.expected_duration = Some(604800);
            }
        }
    }

    /// Optimize schedule
    fn optimize_schedule(&self, plan: &mut ExecutionPlanV1) {
        // Simple schedule optimization: sort tasks by priority
        plan.tasks.sort_by_key(|task| {
            // Extract priority from task name or description if possible
            // For now, just sort by task ID
            task.id.clone()
        });
    }

    /// Collect metrics about the current plan
    fn collect_plan_metrics(&self, plan: &ExecutionPlanV1) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();

        // Total duration
        let total_duration: u64 = plan.tasks.iter().filter_map(|t| t.expected_duration).sum();
        metrics.insert("total_duration".to_string(), total_duration as f64);

        // Task count
        metrics.insert("task_count".to_string(), plan.tasks.len() as f64);

        // Dependency count
        metrics.insert(
            "dependency_count".to_string(),
            plan.dependencies.len() as f64,
        );

        // Average task duration
        if !plan.tasks.is_empty() {
            let avg_duration = total_duration as f64 / plan.tasks.len() as f64;
            metrics.insert("avg_task_duration".to_string(), avg_duration);
        }

        metrics
    }

    /// Calculate planning metadata
    fn calculate_planning_metadata(
        &self,
        plan: &ExecutionPlanV1,
        planning_duration_ms: u64,
    ) -> PlanningMetadata {
        // Calculate critical path length (simplified)
        let critical_path_length: u64 = plan.tasks.iter().filter_map(|t| t.expected_duration).sum();

        // Calculate resource utilization (simplified)
        let mut resource_utilization = HashMap::new();
        if !plan.tasks.is_empty() {
            resource_utilization.insert("compute".to_string(), 0.75); // 75% utilization
            resource_utilization.insert("memory".to_string(), 0.60); // 60% utilization
        }

        PlanningMetadata {
            algorithm_used: "topological-sort".to_string(), // Default for now
            planning_duration_ms,
            task_count: plan.tasks.len(),
            dependency_count: plan.dependencies.len(),
            critical_path_length,
            resource_utilization,
        }
    }

    /// Replan based on changes or new information
    pub fn replan(
        &self,
        original_plan: &ExecutionPlanV1,
        changes: &ReplanningContext,
    ) -> Result<PlannedExecution, PlanningError> {
        // Create new context based on original plan and changes
        let context = PlanningContext {
            goals: changes
                .new_goals
                .clone()
                .unwrap_or_else(|| original_plan.goals.clone()),
            constraints: changes
                .new_constraints
                .clone()
                .unwrap_or_else(|| original_plan.constraints.clone()),
            available_capabilities: changes
                .new_capabilities
                .clone()
                .unwrap_or_else(|| original_plan.required_capabilities.clone()),
            resource_limits: changes.resource_limits.clone().unwrap_or_default(),
            deadline: changes.new_deadline.or_else(|| {
                // Calculate new deadline based on original plan
                Some(original_plan.creation_timestamp + chrono::Duration::days(30))
            }),
            priority: changes.new_priority.unwrap_or(5), // Default priority
        };

        // Create new plan
        let mut result = self.create_plan(context)?;

        // Set parent plan reference
        result.plan.parent_plan_id = Some(original_plan.id.clone());
        result.plan.replan_reason = Some(changes.reason.clone());

        Ok(result)
    }

    /// Get available algorithms
    pub fn algorithms(&self) -> &HashMap<String, PlanningAlgorithm> {
        &self.algorithms
    }

    /// Get optimization strategies
    pub fn optimization_strategies(&self) -> &HashMap<String, OptimizationStrategy> {
        &self.optimization_strategies
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplanningContext {
    pub reason: String,
    pub new_goals: Option<Vec<crate::contracts::execution_plan_v1::Goal>>,
    pub new_constraints: Option<Vec<crate::contracts::execution_plan_v1::Constraint>>,
    pub new_capabilities: Option<Vec<String>>,
    pub new_deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub new_priority: Option<u32>,
    pub resource_limits: Option<HashMap<String, f64>>,
}

#[derive(Debug, thiserror::Error)]
pub enum PlanningError {
    #[error("Invalid planning context: {0}")]
    InvalidContext(String),
    #[error("Planning algorithm failed: {0}")]
    AlgorithmFailure(String),
    #[error("Optimization failed: {0}")]
    OptimizationFailure(String),
}

impl Default for PlanningEngine {
    fn default() -> Self {
        Self::new(ContextEngine::new())
    }
}
