//! Explicitly versioned interfaces for all domain entities

pub mod execution_plan_v1;

pub use execution_plan_v1::ExecutionPlanV1;
pub use execution_plan_v1::{
    Artifact, BackoffStrategy, CompletionCondition, Constraint, ConstraintType, Dependency, Gate,
    GateCriterion, GateType, Goal, Input, ProvenanceInfo, RetryPolicy, Task,
};
