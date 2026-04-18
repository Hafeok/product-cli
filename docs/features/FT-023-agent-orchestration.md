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
domains:
- api
- observability
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
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