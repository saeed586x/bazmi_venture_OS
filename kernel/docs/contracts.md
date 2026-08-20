# Bazmi Kernel Contract Definitions

All Bazmi Kernel contracts are versioned and follow the Canonical Semantic Model.

## Versioned Contracts (v1)

### Core
- `Idea`: Unstructured proposal.
- `Intent`: Formatted goal derived from an `Idea`.
- `Goal`: Measurable objective.
- `Constraint`: Hard boundary or rule.
- `Domain`: Defines the scope.

### Governance
- `Requirement`: Functional/non-functional demand.
- `AcceptanceCriterion`: Validation rule for a `Requirement`.
- `ArchitectureDecision`: Log of an ADR.
- `Risk`: Potential issues.
- `Evidence`: Proof of a result or state.
- `Gate`: Validation point.
- `DecisionRecord`: Immutable log of choices.

### Execution
- `Task`: Atomic unit of work.
- `ExecutionPlan`: Orchestration of `Tasks` to satisfy a `Goal`.
- `Outcome`: Result of execution.

## Relationships
- `Idea` -> `Intent` -> `Goal`
- `Goal` + `Constraints` -> `ExecutionPlan`
- `ExecutionPlan` -> `Task`s -> `Outcome` -> `Evidence` -> `Gate` (Validation)
- `DecisionRecord` links `ArchitectureDecision`s and `Risk`s.
