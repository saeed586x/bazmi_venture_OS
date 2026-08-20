# Work Decomposition Graph

| ID | Workstream | Objective | Dependencies |
|---|---|---|---|
| 1 | Architecture | Define Kernel foundation | - |
| 2 | Contracts | Define versioned interfaces | 1 |
| 3 | Semantic Model | Align with existing schemas | 2 |
| 4 | Governance | Implement policy engine | 2 |
| 5 | Intent/Domain | Implement capability handlers | 3, 4 |
| 6 | PRD/ARD Compilation | Implement compilers | 3, 5 |
| 7 | Validation | Implement verification service | 4, 6 |
| 8 | Planning | Implement DAG orchestration | 5, 7 |
| 9 | Runtime Integration | Implement boundaries | 8 |
| 10 | LLM/MCP Adapters | Connect AI capabilities | 5 |
| 11 | Observability | Implement provenance tracking | 4, 9 |
| 12 | Tests/Hardening | System verification | All |
| 13 | Review | Final audit | 12 |

## Workstream Definitions

### 1. Architecture
- **Objective**: Establish foundation.
- **Inputs**: Repo constraints, requirements.
- **Outputs**: `architecture.md`, design docs.
- **Acceptance Criteria**: Approved design.

... (Other workstreams follow this pattern)
