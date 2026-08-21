# Venture OS Kernel — PR Remediation Work Order

## Machine-readable execution contract

```yaml
work_order_id: venture-os-kernel-pr-remediation
repository: https://github.com/saeed586x/bazmi_venture_OS.git
base_branch: main
pull_request: 1
pull_request_url: https://github.com/saeed586x/bazmi_venture_OS/pull/1
reviewed_head: 92b477647538b6f5516a932efaab6708bf45551a
language: Rust
project_type: Venture OS Kernel
source_of_truth:
  - repository contents
  - .agents/SYSTEM_PROMPT.md
  - .agents/KERNEL_SPECIFICATION.json
merge_policy: DO_NOT_MERGE_UNTIL_ALL_GATES_PASS
completion_policy: NEVER_CLAIM_SUCCESS_WITHOUT_COMMAND_OUTPUT
external_services_policy: interface_tests_are_allowed; real_e2e_requires_real_service
secret_policy: never_request_or_commit_secrets
```

## Global instructions for the coding LLM

You are a senior Rust engineer and verification lead. Work on the existing repository and PR branch. Do not create a parallel project. Read `.agents/SYSTEM_PROMPT.md` and `.agents/KERNEL_SPECIFICATION.json` before modifying code.

For every task below:

1. Inspect the current repository and the current PR diff before editing.
2. Implement the requested behavior; do not replace it with a mock, placeholder, hard-coded success, or documentation-only change.
3. Add focused tests for both success and failure behavior.
4. Run the task-specific verification commands.
5. Report exact files changed, commands run, and command results.
6. Do not claim a test passed if it was not actually run.
7. Do not merge the PR. Do not force-push. Do not rewrite unrelated work.
8. Never place API keys, tokens, passwords, credentials, or private data in source, tests, logs, fixtures, commits, or output.
9. Real external integration may only be claimed if the real service was started and invoked. Otherwise label it `INTERFACE_VERIFIED_ONLY`.
10. Preserve the Kernel's independence from Restate, XState, LLM-provider internals, and MCP internals.
11. `ExecutionPlan.v1` is the boundary contract. Validate it before CLI emission and before adapter submission.
12. LLM output is never authoritative. Canonical contracts and deterministic validation are authoritative.

## Required final status format

Return JSON only when the entire work order is complete:

```json
{
  "status": "PASS | BLOCKED | FAIL",
  "pull_request": 1,
  "head_commit": "<actual commit SHA>",
  "tasks": {
    "ISSUE-01": {"status": "PASS | FAIL | BLOCKED", "evidence": []}
  },
  "quality_gates": {
    "fmt": "PASS | FAIL",
    "check": "PASS | FAIL",
    "test": "PASS | FAIL",
    "clippy": "PASS | FAIL"
  },
  "test_count": 0,
  "external_e2e": "VERIFIED | INTERFACE_VERIFIED_ONLY | NOT_RUN",
  "merge_recommendation": "APPROVE | REQUEST_CHANGES | DO_NOT_MERGE",
  "blockers": [],
  "changed_files": [],
  "security_scan": "PASS | FAIL"
}
```

If any required gate fails, use `REQUEST_CHANGES` or `DO_NOT_MERGE`; never use `APPROVE`.

---

# ISSUE-00 — Establish a reproducible baseline

```yaml
id: ISSUE-00
priority: P0
independent: true
blocking: true
```

## Objective

Establish the actual starting state. The previous PR claim that all quality gates passed is false because the workspace test suite failed in the LLM adapter test.

## Required actions

Run and record:

```bash
pwd
git status --short --branch
git branch --show-current
git log --oneline -10
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
grep -RInE '(TODO|FIXME|mock|stub|placeholder|not implemented|In a real)' src tests || true
grep -RInE '(api[_-]?key|secret|token|password|Bearer|ghp_|sk-|OPENAI|ANTHROPIC|GEMINI)' . --exclude-dir=.git --exclude-dir=target || true
git ls-files | grep -E '(^target/|\.env($|\.)|\.log$|\.pem$|\.key$)' || true
```

Do not modify code in this issue unless required to make the baseline reproducible. Save the complete output as review evidence.

## Acceptance criteria

- The baseline is recorded.
- The known Tokio reactor failure is reproduced or its current replacement is identified.
- No claim of green quality gates is made unless all four commands genuinely pass.

---

# ISSUE-01 — Fix the asynchronous LLM test and make adapter tests deterministic

```yaml
id: ISSUE-01
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-00]
```

## Known failure

`cargo test --workspace` fails in `tests/e2e_integration_test.rs::test_llm_adapter_basic_functionality` because `futures::executor::block_on` is used with a Reqwest client that requires a Tokio reactor:

```text
there is no reactor running, must be called from the context of a Tokio 1.x runtime
```

## Required implementation

1. Run async adapter tests inside Tokio, using either:
   - `#[tokio::test]`, or
   - an explicitly created Tokio runtime.
2. Do not use `futures::executor::block_on` for Reqwest/Tokio network operations.
3. Do not make tests depend on public LLM providers, credentials, internet access, or nondeterministic responses.
4. Add a local mock HTTP server or an equivalent deterministic test transport.
5. Test at least:
   - successful OpenAI-shaped response parsing;
   - successful Anthropic-shaped response parsing;
   - successful Google-shaped response parsing;
   - non-success HTTP status handling;
   - malformed response handling;
   - missing credentials or authentication failure behavior;
   - timeout/network failure behavior.
6. Ensure test output never prints credentials or authorization headers.

## Acceptance criteria

```yaml
required:
  - cargo test --workspace passes
  - no Tokio reactor panic
  - no real provider credentials required
  - provider response parsing is covered by deterministic tests
  - failure paths return typed LLM errors
```

## Verification

```bash
cargo test --workspace
cargo test --test e2e_integration_test test_llm_adapter_basic_functionality -- --exact --nocapture
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

# ISSUE-02 — Make `Kernel::process_intent` contract-valid and registry-aware

```yaml
id: ISSUE-02
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-00]
```

## Objective

Implement real deterministic intent-to-plan orchestration while preserving the `ExecutionPlan.v1` boundary contract.

## Required behavior

For a non-empty intent, `Kernel::process_intent` must produce a plan with:

```yaml
goals: exactly 3
constraints: exactly 2
required_capabilities: exactly 4
tasks: exactly 4
dependencies: at_least 1
 gates: exactly 2
completion_conditions: exactly 3
provenance: non-null
id: UUID
version: valid semantic version
creation_timestamp: valid ISO-8601 timestamp
```

The four task dependencies must form an acyclic DAG. Every dependency must reference an existing task ID. Every task must have a capability. Every required capability must be registered and enabled in `CapabilityRegistry`.

## Contract corrections required

1. Decide and implement the specification-compliant representation of `intent_reference`.
   - Prefer a stable UUID reference to a persisted/constructed `Idea` or `Intent` contract.
   - Do not silently use arbitrary raw natural-language text where the contract requires a UUID.
2. Provenance must contain meaningful origin, transformation/reasoning metadata, confidence, and auditability.
3. Add independent `ExecutionPlan.v1::validate()` validation, or strengthen the existing validator, to reject:
   - invalid UUIDs;
   - invalid semantic versions;
   - empty required collections;
   - unknown task dependency references;
   - cyclic dependencies;
   - empty gate criteria;
   - missing provenance;
   - replans without `parent_plan_id` and `replan_reason` pairing.
4. Ensure the CLI serializes only a plan that has passed contract validation.
5. Preserve raw intent in a separate appropriate field or provenance/evidence structure if it is needed for traceability.

## Required tests

- Empty intent is rejected.
- Whitespace-only intent is rejected.
- Normal intent produces the required non-empty plan.
- All four capabilities resolve through the registry.
- Dependency references are valid.
- Cyclic dependencies are rejected.
- Plan serializes and deserializes independently with Serde.
- Invalid plans are rejected with specific violations.
- Provenance is present and meaningful.

## Verification

```bash
cargo test --workspace
cargo run --quiet --bin venture-cli -- 'Build a customer portal' > /tmp/execution-plan.json
python3 - <<'PY'
import json
p = json.load(open('/tmp/execution-plan.json'))
assert len(p['goals']) == 3
assert len(p['constraints']) == 2
assert len(p['required_capabilities']) == 4
assert len(p['tasks']) == 4
assert len(p['dependencies']) >= 1
assert len(p['gates']) == 2
assert len(p['completion_conditions']) == 3
assert p['provenance'] is not None
assert 'mock-123' not in open('/tmp/execution-plan.json').read()
print('ExecutionPlan.v1 checks passed')
PY
```

---

# ISSUE-03 — Correct and test the CLI boundary

```yaml
id: ISSUE-03
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-02]
```

## Required behavior

`venture-cli` must:

1. Call the real `Kernel::process_intent`.
2. Emit exactly one valid JSON `ExecutionPlan.v1` document to stdout on success.
3. Emit diagnostics only to stderr.
4. Never emit `mock-123`, `Mock execution plan generated`, or any fake plan text.
5. Exit non-zero for missing or invalid input.
6. Reject or correctly handle intents containing arbitrary Unicode and JSON-sensitive characters.
7. Serialize a contract-validated plan.

## Required tests

- Valid idea emits parseable JSON.
- Output has all mandatory `ExecutionPlan.v1` fields.
- Output does not contain `mock-123`.
- Missing argument exits non-zero.
- Invalid intent exits non-zero.
- Stdout remains machine-readable JSON.

## Verification

```bash
cargo test --workspace
cargo run --quiet --bin venture-cli -- 'Create a secure customer portal' > /tmp/cli-output.json
python3 -m json.tool /tmp/cli-output.json >/dev/null
test "$(grep -c 'mock-123' /tmp/cli-output.json)" -eq 0
```

---

# ISSUE-04 — Make the LLM adapter provider-neutral, correct, and testable

```yaml
id: ISSUE-04
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-01]
```

## Scope

Review `src/runtime/llm_adapter.rs`. The PR adds HTTP calls, but the implementation needs stronger provider correctness and deterministic tests.

## Required implementation

1. Keep the Kernel independent of provider-specific APIs.
2. Use an injectable HTTP client or test transport where practical.
3. Support provider-specific request and response shapes for OpenAI, Anthropic, and Google.
4. Ensure Google authentication is actually sent in the provider-required way when an API key is configured.
5. Do not log API keys or authorization headers.
6. Apply configured timeout behavior.
7. Return typed errors for:
   - missing credentials where credentials are required;
   - authentication failures;
   - non-success HTTP statuses;
   - malformed JSON;
   - provider response schema mismatch;
   - network errors;
   - timeout errors.
8. Do not return fabricated text when an HTTP call fails.
9. Ensure `process_structured` validates or clearly treats the provider response as untrusted reasoning output; it must not become canonical truth automatically.
10. Add request-shape and response-shape tests using a local test server. Do not use production provider credentials.

## Acceptance criteria

```yaml
provider_neutral_interface: true
mock_success_without_http_call: false
secret_logging: false
real_external_provider_required_for_tests: false
provider_shapes_tested: [OpenAI, Anthropic, Google]
```

## Verification

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

# ISSUE-05 — Correct the Restate boundary without coupling the Kernel to Restate internals

```yaml
id: ISSUE-05
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-02]
```

## Scope

Review `src/runtime/restate_adapter.rs`. The adapter must remain a boundary adapter. The Kernel emits `ExecutionPlan.v1`; Restate owns durable execution.

## Required implementation

1. Validate `ExecutionPlan.v1` before submission.
2. Use an explicit, documented request contract for the configured Restate endpoint.
3. Do not invent a claim of real Restate compatibility unless the endpoint format is verified against Restate documentation or a running Restate server.
4. Preserve idempotency for repeated submissions of the same plan ID.
5. Apply timeout and authentication configuration without exposing secrets.
6. Return typed errors for HTTP, status, JSON, authentication, and invalid-plan failures.
7. Parse execution ID and status only from validated response fields; do not silently fabricate success.
8. Add local mock-server tests for:
   - successful submission;
   - server rejection;
   - malformed response;
   - status lookup;
   - timeout/network failure;
   - invalid plan rejected before network submission.
9. If a real Restate server is available, run an external integration test and record the exact server/version and invocation. Otherwise report `INTERFACE_VERIFIED_ONLY`.

## Verification

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

# ISSUE-06 — Correct the MCP adapter and retry semantics

```yaml
id: ISSUE-06
priority: P1
independent: true
blocking: true
depends_on: [ISSUE-00]
```

## Scope

Review `src/runtime/mcp_adapter.rs`.

## Required implementation

1. Define the supported MCP boundary explicitly. If this is an HTTP tool gateway rather than the MCP protocol itself, rename/document it accurately.
2. Do not claim MCP protocol compatibility without protocol-level verification.
3. Validate tool existence, enabled state, required parameters, and parameter types before invocation.
4. Use an idempotency/execution identifier when supported by the endpoint.
5. Implement `max_retries` with bounded retry behavior for explicitly retryable failures only.
6. Do not retry authentication failures, malformed requests, or non-retryable tool errors.
7. Apply timeout configuration.
8. Return typed errors and preserve the tool's response without fabricating success.
9. Add local mock-server tests for:
   - successful invocation;
   - missing tool;
   - disabled tool;
   - missing required parameter;
   - invalid parameter type;
   - retryable 5xx response followed by success;
   - non-retryable 4xx response;
   - timeout/network error;
   - response parsing.
10. Never log credentials.

## Verification

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

# ISSUE-07 — Make governance policy evaluation deterministic and unambiguous

```yaml
id: ISSUE-07
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-00]
```

## Scope

Review `src/core/governance.rs`.

## Required behavior

Define the policy semantics explicitly. A rule must state whether its condition is:

- a condition that must be true for compliance; or
- a forbidden condition that triggers a violation.

Do not leave this ambiguous. Preserve compatibility with existing policy data or migrate it deliberately.

Support deterministic operators:

```yaml
operators: [starts_with, contains, ends_with, ==, !=]
```

## Required tests

For every operator, test both matching and non-matching values. Also test:

- disabled policies are ignored;
- multiple policies and rules aggregate violations;
- severity is preserved;
- malformed/unknown rule syntax has defined behavior;
- policy evaluation is deterministic and side-effect free;
- `validate_plan` applies the same semantics as `validate`.

Unknown syntax must not silently authorize a dangerous action. Choose and document either rejection or an explicit non-enforcing advisory result.

## Verification

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

# ISSUE-08 — Complete or explicitly classify remaining production stubs

```yaml
id: ISSUE-08
priority: P0
independent: true
blocking: true
depends_on: [ISSUE-00]
```

## Known locations

Search all production code for placeholder behavior, including:

```text
src/runtime/observability.rs
src/capabilities/decision_memory.rs
src/capabilities/verification_engine.rs
src/capabilities/validation_engine.rs
src/capabilities/requirements_engine.rs
src/capabilities/planning_engine.rs
src/capabilities/intent_engine.rs
src/capabilities/domain_engine.rs
src/capabilities/decision_gateway.rs
src/runtime/xstate_adapter.rs
```

## Required action

For every placeholder, choose exactly one:

1. Implement real deterministic behavior and add tests; or
2. Move it behind an explicit experimental feature/boundary and document that it is not production behavior; or
3. Remove dead code.

Do not leave comments such as `In a real implementation` where production behavior is required. Do not replace incomplete behavior with fake success.

At minimum, inspect and address:

- observability file and remote destinations currently printing instead of writing/sending;
- observability quantile calculation if exposed as a real metric;
- decision-memory similarity behavior;
- validation custom-rule evaluation;
- verification-rule evaluation and simulation success determination;
- requirements rule evaluation and traceability;
- planning DAG validation and optimization claims;
- intent extraction claims versus actual deterministic behavior;
- domain validation and bounded-context invariants;
- decision gateway route and gate enforcement behavior;
- XState XML escaping and valid SCXML output.

## Acceptance criteria

Every remaining placeholder is either removed, implemented, or explicitly classified as non-production. The final report must list each one and its disposition.

---

# ISSUE-09 — Strengthen XState boundary output

```yaml
id: ISSUE-09
priority: P1
independent: true
blocking: true
depends_on: [ISSUE-02]
```

## Required implementation

1. Generate states and transitions from the plan without invalid references.
2. Produce syntactically valid JSON, SCXML, and Mermaid output where those formats are advertised.
3. Escape XML attribute values and Mermaid-sensitive values.
4. Ensure SCXML has exactly one valid closing `</scxml>` tag and valid transition syntax.
5. Add tests for:
   - empty and non-empty plans;
   - task dependency transitions;
   - gate/completion states;
   - special characters in IDs/names/guards;
   - JSON parseability;
   - SCXML structural validity.
6. Keep XState as an adapter boundary; do not import an XState runtime into the Kernel.

## Verification

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

---

# ISSUE-10 — Add contract, integration, and security coverage

```yaml
id: ISSUE-10
priority: P0
independent: false
blocking: true
depends_on: [ISSUE-01, ISSUE-02, ISSUE-03, ISSUE-04, ISSUE-05, ISSUE-06, ISSUE-07, ISSUE-08, ISSUE-09]
```

## Required tests

Add meaningful tests for:

- Kernel process flow;
- non-empty `ExecutionPlan.v1`;
- independent serialization/deserialization;
- invalid contract rejection;
- capability registration, lookup, duplicate prevention, and dependency resolution;
- intent extraction and validation;
- domain modeling and invariants;
- requirements and acceptance criteria;
- PRD and ARD traceability;
- governance and gate enforcement;
- validation violations;
- deterministic simulation/verification;
- risk scoring as `probability * impact`;
- decision routing and decision records;
- provenance/audit trail;
- decision memory storage/retrieval;
- DAG dependency validation and cycle rejection;
- dynamic replanning with incremented semantic plan version, `parent_plan_id`, and `replan_reason`;
- LLM, MCP, Restate, and XState boundaries;
- CLI JSON output;
- secret scan and absence of tracked build artifacts.

Tests must be meaningful. Do not inflate the count with empty assertions or construction-only tests. If fewer than 100 meaningful tests exist, report `test_count` accurately and do not claim production readiness.

---

# ISSUE-11 — Run final quality and security gates

```yaml
id: ISSUE-11
priority: P0
blocking: true
depends_on: [ISSUE-10]
```

## Required commands

Run exactly:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
git status --short
git diff --stat
git ls-files | grep -E '(^target/|\.env($|\.)|\.log$|\.pem$|\.key$)' || true
grep -RInE '(api[_-]?key|secret|token|password|Bearer|ghp_|sk-|OPENAI|ANTHROPIC|GEMINI)' . --exclude-dir=.git --exclude-dir=target || true
```

## Acceptance criteria

All four Rust quality gates pass. No build artifacts or secrets are tracked. The working tree is clean except for intentional source changes. Any external service not actually started and invoked is reported as `INTERFACE_VERIFIED_ONLY`.

---

# ISSUE-12 — Final PR decision and evidence

```yaml
id: ISSUE-12
priority: P0
blocking: true
depends_on: [ISSUE-11]
```

## Required final report

Return machine-readable JSON using the global format. Include:

- actual PR head SHA;
- each issue status;
- exact test count;
- exact quality-gate results;
- changed files;
- unresolved placeholders and their classification;
- external integration status;
- security scan result;
- blockers;
- merge recommendation.

The merge recommendation must be:

```yaml
APPROVE:
  allowed_only_if:
    - all required quality gates pass
    - all blocking issue acceptance criteria pass
    - no unreviewed production stubs remain
    - no secrets or build artifacts are tracked
    - no false external E2E claim is made
REQUEST_CHANGES:
  required_if:
    - any issue is incomplete
    - any quality gate fails
    - any claim is unsupported by command output
DO_NOT_MERGE:
  required_if:
    - tests fail
    - contract validation is bypassed
    - fake/mock success remains in required production paths
    - secrets are found
```

Do not merge this PR as part of this work order. First produce the evidence and recommendation.
