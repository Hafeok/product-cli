---
id: FT-023
title: Agent Orchestration
phase: 5
status: complete
depends-on: []
adrs:
- ADR-021
- ADR-035
tests:
- TC-108
- TC-109
- TC-110
- TC-111
- TC-112
- TC-113
- TC-114
- TC-115
- TC-167
- TC-304
- TC-305
- TC-306
- TC-307
- TC-309
- TC-310
- TC-311
- TC-312
- TC-313
- TC-314
- TC-712
- TC-713
- TC-714
- TC-715
domains:
- api
- observability
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

### `product implement FT-XXX`

The implementation command runs a five-step pipeline:

**Step 1 — Gap gate.** Runs `product gap check FT-XXX`. If any high-severity gaps (G001, G002, G005) are found and unsuppressed, the command exits with an explanation. You cannot implement a specification with known high-severity gaps — the agent would be working from an incomplete contract.

**Step 2 — Drift check.** Runs `product drift check --phase N` for the feature's phase. If the codebase has already drifted from a related ADR, the agent needs to know before it writes more code.

**Step 3 — Context assembly.** Runs `product context FT-XXX --depth 2`. Wraps it in the versioned implementation prompt from `benchmarks/prompts/implement-v1.md`.

**Step 4 — Agent invocation.** Invokes the configured agent with the assembled context. For Claude Code: pipes the context bundle to `claude --print` or writes it to a temp file and passes the file path.

**Step 5 — Auto-verify.** On agent completion, runs `product verify FT-XXX` automatically unless `--no-verify` is passed.

```
product implement FT-001
  ✓ Gap check: no high-severity gaps
  ✓ Drift check: no drift detected
  → Assembling context bundle (FT-001, 4 ADRs, 6 TCs, depth 2)
  → Invoking claude-code...
  [agent output streams here]
  → Running product verify FT-001...
  TC-001 binary-compiles         PASS
  TC-002 raft-leader-election    PASS
  TC-003 raft-leader-failover    FAIL
  ✗ 1 test failing. Feature status: in-progress
```

### `product verify FT-XXX`

Verify reads each linked TC file and derives how to run it from the TC metadata:

```yaml
---
id: TC-002
type: scenario
runner: cargo-test
runner-args: ["--test", "raft_leader_election", "--", "--nocapture"]
---
```

The `runner` and `runner-args` fields in TC front-matter tell verify how to execute the criterion. Supported runners: `cargo-test`, `bash`, `pytest`, `custom`.

On completion:
- TC statuses updated (`passing`, `failing`)
- Feature status updated if all TCs pass → `complete`
- `checklist.md` regenerated
- Results written to stdout in the error model format (ADR-013)

### Implementation Prompt

The implementation prompt wraps the context bundle with explicit constraints:

```markdown
# Implementation Task: {FEATURE_ID} — {FEATURE_TITLE}

```

---

## Description

Agent orchestration covers the two commands that sit at the boundary between Product's knowledge surface and agent work: `product implement FT-XXX` (the pre-implementation pipeline of gap gate, drift check, context assembly, and auto-verify) and `product verify FT-XXX` (TC execution, TC status update, feature status update, and checklist regeneration). Product is a knowledge tool — it does not invoke agents directly (ADR-021). The implementation pipeline is a sequence of knowledge commands that a harness calls; `product verify` is the one pipeline command Product owns because it writes back to the graph.

## Functional Specification

### Inputs

- **`product implement FT-XXX`**: feature ID; optional `--no-verify` to skip auto-verify; optional `--no-auto-runners` to skip runner config auto-fill (Step 0a); optional `--dry-run` to print the plan without writing
- **`product verify FT-XXX`**: feature ID; `--platform` to run all cross-cutting TCs regardless of feature association
- **TC front-matter fields**: `runner` (cargo-test | bash | pytest | custom), `runner-args`, optional `runner-timeout` (default 30s), optional `requires` (declarative prerequisites)
- **`product.toml` `[verify.prerequisites]`**: named shell commands checked before running a TC with a `requires` field
- **`gaps.json`**: baseline file checked by the gap gate — suppressed findings do not block implementation

### Outputs

- **`product implement`**: gap/drift findings to stdout (if any block), then context bundle path; on agent completion, delegates to `product verify`
- **`product verify`**: per-TC status lines (`PASS` / `FAIL` / `UNIMPLEMENTED` / `UNRUNNABLE`) on stdout; TC front-matter fields updated (`status`, `last-run`, `last-run-duration`, `failure-message`); feature status updated if all runnable TCs pass; `CHECKLIST.md` regenerated
- **Exit codes** follow ADR-009: 0 = clean, 1 = errors, 2 = warnings only

### State

- TC statuses are written back to each TC's YAML front-matter by `product verify` (atomic writes per ADR-015).
- Feature status is written back to the feature's front-matter when all runnable TCs pass (`complete`) or any runnable TC fails (`in-progress`).
- `CHECKLIST.md` is regenerated by `product checklist generate` after every `product verify` run.
- `gaps.json` is read (not written) by the gap gate; suppressions are managed by `product gap suppress`.

### Behaviour

**`product implement FT-XXX` pipeline:**

1. **Step 0a — Auto-fill runners**: for every TC linked to the feature that lacks `runner` or `runner-args`, derive `runner: cargo-test`, `runner-args: tc_<NNN>_<slug>`, `runner-timeout: 120s` from the TC filename and write atomically. Skipped if `--no-auto-runners` is passed.
2. **Step 0 — Preflight**: runs `product preflight FT-XXX`. Domain coverage gaps block implementation and cannot be bypassed — they must be linked or acknowledged.
3. **Step 1 — Gap gate**: runs `product gap check FT-XXX --severity high`. Any unsuppressed high-severity gap (G001, G002, G005) exits 1 with E009.
4. **Step 2 — Drift check**: runs `product drift check --phase N` for the feature's phase. Drift is advisory; the pipeline continues regardless.
5. **Step 3 — Context assembly**: runs `product context FT-XXX --depth 2` and wraps it in the implementation prompt from `benchmarks/prompts/implement-v1.md`.
6. **Step 4 — Agent output**: the assembled bundle is available for the harness to pass to any agent. Product does not invoke the agent.
7. **Step 5 — Auto-verify**: on agent completion, runs `product verify FT-XXX` unless `--no-verify` is passed.

**`product verify FT-XXX` behaviour:**

- For each linked TC: if `runner` is missing and feature is `in-progress` or `complete`, fail with E022. If a `requires` prerequisite is not satisfied, mark TC `unrunnable`. Otherwise, run the configured command and record exit code.
- All runnable TCs pass → feature status → `complete`. Any runnable TC fails → feature status → `in-progress`. All TCs unrunnable → status unchanged, W-class warning.
- TC front-matter is updated: `status`, `last-run`, `last-run-duration`, `failure-message` (on fail or unrunnable).
- `product checklist generate` runs automatically after status updates.

### Invariants

- TCs without `runner` or `runner-args` fail with E022 at five gates — `product preflight`, `product request apply`, `product feature status …in-progress`, `product graph check`, and `product verify` — when the linked feature is `in-progress` or `complete`. For `planned` or `abandoned` features the soft-skip behaviour is preserved.
- Product never manages test setup, teardown, or infrastructure. The runner boundary is exactly: call the configured command, wait for exit, record the result. Everything inside the command is the test suite's responsibility (ADR-021).
- The `requires` field is evaluated (read-only condition check) but never satisfied by Product. Product reports `unrunnable` with the prerequisite name; satisfying prerequisites is the developer's responsibility.
- `product implement` does not invoke agents. Harness scripts in `scripts/harness/` illustrate composition but are not part of Product's binary.

### Error handling

- **E009**: high-severity gap found during gap gate — exits 1 with gap details. Resolve or suppress the gap to proceed.
- **E022**: TC runner configuration missing for an active feature — exits 1 listing all offending TCs.
- **Preflight failures**: domain coverage gaps block `product implement`; unlike the gap gate, they cannot be suppressed (only acknowledged with explicit reasoning).
- **Runner exit non-zero**: TC marked `failing`; failure message from stdout/stderr captured into `failure-message` front-matter field.
- **Unrunnable prerequisite**: TC marked `unrunnable` with prerequisite name; feature status not blocked by unrunnable TCs.
- **Model errors in drift check**: drift is advisory; a model error produces a stderr warning and the pipeline continues.

### Boundaries

- Product does not invoke agents, select agents, or manage agent context windows.
- Product does not manage test fixtures, database state, cluster initialisation, or any test infrastructure. Wrapper scripts are the correct escape hatch for TCs that require environment setup.
- The `product implement` pipeline is a convenience sequence of knowledge commands — each command in the sequence is independently invocable by a harness without using `product implement`.

## Out of scope

- Agent invocation (the harness's responsibility, not Product's)
- Test fixture management or test infrastructure setup/teardown
- Runner selection based on language or framework (the TC author configures the runner)
- Retry logic for flaky tests (the harness decides on retry policy)
- Parallel TC execution (TCs run sequentially in `product verify`)
