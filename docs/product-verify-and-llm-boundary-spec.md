# Product Verify Pipeline and LLM Boundary Specification

> Two changes specified together because they are the same architectural decision:
> (1) what `product verify` runs, and (2) what Product never does.
>
> Covers: ADR-033 (unified verify pipeline) and amendments to ADR-019, ADR-022,
> ADR-023, and product.toml removing all LLM call sites from Product.

---

## The Principle

Product is a knowledge tool. It assembles, validates, and presents information.
It does not invoke LLMs.

This principle was established in ADR-021 when `product implement` (which spawned
agents) was removed. The same principle applies to `product gap check`,
`product drift check`, and `product adr review --staged` — all of which currently
make LLM API calls. Product assembling a context bundle and piping it to an LLM is
orchestration. Product assembling the bundle and writing it to stdout is knowledge
provision. The user decides what to do with it.

**Before:**
```bash
product gap check ADR-002          # ← calls LLM internally
product drift check FT-001         # ← calls LLM internally
product adr review --staged        # ← calls LLM internally
```

**After:**
```bash
product gap bundle ADR-002 | claude "find gaps"     # user wires LLM
product drift diff FT-001  | claude "find drift"    # user wires LLM
product adr review --staged                         # structural checks only, instant
```

Product produces the right input for an LLM call. The LLM call is the user's concern.

---

## ADR-033: Unified Verify Pipeline

### Status

Accepted

### Context

Product has multiple verification commands that must be run in a specific order to
produce a meaningful result. Currently a developer or CI script must know and manually
sequence: `product request log verify`, `product graph check`, `product metrics
threshold`, `product verify FT-XXX` per feature, `product verify --platform`. Each
command has its own exit code semantics and output format.

There is no single entry point that says "is this repository in a good state?" The
closest analogy — `dotnet build` for a solution, `cargo check` for a workspace — is
missing.

`product verify` with no arguments becomes that entry point: run every structural,
metric, and test check in the correct order, produce a structured report, exit with
the worst result.

### Decision

`product verify` with no arguments runs the full verification pipeline. The pipeline
has six stages, ordered by cost and dependency:

```
Stage 1  Log integrity
Stage 2  Graph structure
Stage 3  Schema validation
Stage 4  Metrics thresholds
Stage 5  Feature TCs
Stage 6  Platform TCs
```

LLM-dependent checks (gap analysis, drift detection) are removed from the pipeline.
They are available as separate commands that produce LLM-ready output for the user
to pipe as they choose.

Each stage runs regardless of whether earlier stages failed — the pipeline always
produces a complete picture. Exit code is the worst result across all stages:
0 (all pass), 1 (any E-class error), 2 (warnings only).

---

### Pipeline Stages

#### Stage 1 — Log Integrity

Runs `product request log verify`.

Checks: entry hash validity, chain integrity, cross-reference against git tags.

| Result | Condition |
|---|---|
| pass | All hashes valid, chain intact, tags cross-reference clean |
| error (exit 1) | E015 (hash mismatch) or E016 (chain break) |
| warning (exit 2) | W021 (tag without log entry) |

Cost: O(N) where N = log entries. Fast for any realistic log size (<1s).

---

#### Stage 2 — Graph Structure

Runs `product graph check`.

Checks: broken links (E002), dependency cycles (E003), supersession cycles (E004),
malformed front-matter (E001), domain vocabulary (E012), missing ADR governance (E013),
all W-class structural warnings (W001–W017, W020).

| Result | Condition |
|---|---|
| pass | Zero E-class findings |
| error (exit 1) | Any E-class finding |
| warning (exit 2) | W-class findings only |

Cost: O(V+E) on the graph. Fast for any realistic repository (<500ms).

---

#### Stage 3 — Schema Validation

Validates `schema-version` in `product.toml` against the binary's supported version.

| Result | Condition |
|---|---|
| pass | Schema version compatible |
| error (exit 1) | E008 (schema version ahead of binary) |
| warning (exit 2) | W007 (upgrade available) |

Cost: instant.

---

#### Stage 4 — Metrics Thresholds

Runs `product metrics threshold`.

Checks all configured thresholds in `[metrics.thresholds]` against current metrics.

| Result | Condition |
|---|---|
| pass | All thresholds satisfied |
| error (exit 1) | Any `severity = "error"` threshold breached |
| warning (exit 2) | Any `severity = "warning"` threshold breached |

Cost: reads `metrics.jsonl` and front-matter bundle blocks. Fast (<200ms).

---

#### Stage 5 — Feature TCs

For each feature in topological order:
1. Skip if `status: planned` (no implementation yet)
2. Run `product verify FT-XXX` — all linked TCs
3. Record pass/fail/unrunnable per TC

Stage 5 only runs features reachable from the current phase gate. Features in locked
phases are skipped with a note.

| Result | Condition |
|---|---|
| pass | All runnable TCs passing across all in-scope features |
| error (exit 1) | Any TC failing |
| warning (exit 2) | Any TC unrunnable (but none failing) |

Cost: depends on test suite. Dominates pipeline wall time for large repositories.

---

#### Stage 6 — Platform TCs

Runs `product verify --platform` — TCs linked to cross-cutting ADRs (TC-CQ-001
through TC-CQ-005 and others).

| Result | Condition |
|---|---|
| pass | All platform TCs passing |
| error (exit 1) | Any platform TC failing |
| warning (exit 2) | Any platform TC unrunnable |

Cost: includes code quality checks (file length, function length, module structure).
Fast for the structural checks; depends on any custom platform TCs.

---

### Output Format

```
product verify

  [1/6] Log integrity .............. ✓  clean (52 entries, chain intact)
  [2/6] Graph structure ............ ⚠  3 warnings
              W012  FT-013 has no bundle measurement
              W016  FT-002 has 1 unimplemented TC
              W017  FT-001 spec changed since completion
  [3/6] Schema validation .......... ✓  clean
  [4/6] Metrics thresholds ......... ⚠  1 warning
              bundle_tokens_p95: 10,800  (threshold: 10,000)
  [5/6] Feature TCs ................ ✗  2 failing
              TC-007  raft-leader-failover    FAIL  FT-003  (18.1s)
              TC-012  volume-allocation-e2e   FAIL  FT-004  (32.4s)
              TC-050  rate-limit-100rps       SKIP  FT-009  [phase 2 locked]
  [6/6] Platform TCs ............... ✓  5/5 passing

  ────────────────────────────────────────────────────────────────
  Result:  FAIL  (2 TCs failing)
  Exit:    1

  Features needing attention:
    FT-001  complete    ⚠  W017 spec changed — run: product verify FT-001
    FT-002  complete    ⚠  W016 unimplemented TC
    FT-003  in-progress ✗  TC-007 failing
    FT-004  in-progress ✗  TC-012 failing
```

### Scope Flags

```bash
product verify                      # everything — all phases, all features
product verify --phase 1            # scope to phase 1 features only
product verify FT-001               # scope to one feature (current behaviour)
product verify --ci                 # structured JSON output, no colour
```

`product verify FT-XXX` remains the per-feature command. Its behaviour is unchanged:
run TCs for that feature, update status, create git tag on completion. The only
change is that it is now also stage 5 of the full pipeline when called without a
feature argument.

### `--ci` Flag

Writes structured JSON to stdout for pipeline integration:

```json
{
  "passed": false,
  "exit": 1,
  "stages": [
    { "stage": 1, "name": "log-integrity",     "status": "pass",    "findings": [] },
    { "stage": 2, "name": "graph-structure",   "status": "warning", "findings": ["W012", "W016", "W017"] },
    { "stage": 3, "name": "schema-validation", "status": "pass",    "findings": [] },
    { "stage": 4, "name": "metrics",           "status": "warning", "findings": ["bundle_tokens_p95"] },
    { "stage": 5, "name": "feature-tcs",       "status": "fail",    "findings": [
        { "tc": "TC-007", "feature": "FT-003", "status": "failing" },
        { "tc": "TC-012", "feature": "FT-004", "status": "failing" }
    ]},
    { "stage": 6, "name": "platform-tcs",      "status": "pass",    "findings": [] }
  ]
}
```

### Test Coverage

Session tests:

```
ST-110  verify-all-pass-clean-repo
ST-111  verify-fails-on-e-class-graph-error
ST-112  verify-warns-on-w-class-only
ST-113  verify-fails-on-failing-tc
ST-114  verify-skips-locked-phase-features
ST-115  verify-phase-scope-flag
ST-116  verify-ci-json-output
ST-117  verify-feature-scope-unchanged     # product verify FT-001 still works
ST-118  verify-log-integrity-stage-1       # tampered log → stage 1 fails
ST-119  verify-metrics-threshold-stage-4  # threshold breached → stage 4 warns
```

---

## LLM Boundary Amendments

### Amendment to ADR-019: Gap Analysis

**Previous decision:** `product gap check ADR-XXX` called an LLM internally.

**Amended decision:** `product gap bundle ADR-XXX` assembles the gap-check input and
writes it to stdout. No LLM call. The user pipes to their LLM.

#### New command: `product gap bundle`

```bash
product gap bundle ADR-002              # gap-check input for one ADR
product gap bundle --changed            # gap-check inputs for ADRs changed since last run
product gap bundle --all                # gap-check inputs for all ADRs
product gap bundle ADR-002 --format json  # machine-readable
```

Output is a markdown document structured for LLM consumption:

```markdown
# Gap Analysis Input: ADR-002 — openraft for Cluster Consensus

## Instructions

You are performing gap analysis on an architectural decision record.
Check for the following gap types only. For each gap found, output a
JSON object with fields: code, severity, description, location.

Gap types to check:
- G001: Testable claim with no linked TC
- G002: Formal invariant block with no scenario/chaos TC
- G003: No rejected alternatives section
- G004: Rationale references uncaptured external constraint
- G005: Logical inconsistency with a linked ADR
- G006: Feature aspect not addressed by any linked ADR
- G007: Rationale references superseded decisions
- G008: Feature uses dependency with no governing ADR

Output format: one JSON object per line, nothing else.

## Context Bundle

[full depth-2 context bundle for ADR-002]
```

The output is the same context bundle that was previously sent to the LLM internally
— now written to stdout for the user to direct as they choose.

#### What happens to `product gap check`?

`product gap check` becomes a structural-only command. It no longer calls an LLM.
It checks the graph for mechanically detectable gap indicators:

| Gap code | Structural equivalent | Check |
|---|---|---|
| G001 | ADR has testable language patterns with no TC linked | Heuristic keyword scan (threshold, must, always, never, exactly) — advisory |
| G002 | `⟦Γ:Invariants⟧` block present, no scenario/chaos TC linked | Structural — deterministic |
| G003 | No rejected alternatives section in ADR body | Structural — deterministic |
| G008 | Feature uses DEP with no governing ADR | Structural — deterministic (E013) |

G004, G005, G006, G007 require semantic understanding — they are not checkable
structurally and are removed from `product gap check`. They remain as gap types
documented in the prompt template in `benchmarks/prompts/gap-analysis-v1.md`,
which the user passes to their LLM via `product gap bundle`.

`product gap check` is fast, deterministic, and never calls an LLM.
`product gap bundle | your-llm` is the semantic analysis path.

#### `gaps.json` and suppression

The suppression model remains. Structural gap findings from `product gap check`
continue to use `gaps.json`. LLM-detected gaps from the user's analysis are outside
Product's scope — the user manages them in whatever tool they choose.

#### `product.toml` — removed config

```toml
# REMOVED — Product no longer calls an LLM for gap analysis
[gap-analysis]
# prompt-version = "1"   ← now lives in benchmarks/prompts/gap-analysis-v1.md
# model = "claude-sonnet-4-6"
# max-findings-per-adr = 10
# severity-threshold = "medium"
```

The prompt file remains at `benchmarks/prompts/gap-analysis-v1.md` as a resource
the user passes to their LLM. Product manages its versioning via
`product prompts list/get/update` — the same mechanism as authoring prompts.

---

### Amendment to ADR-023: Drift Detection

**Previous decision:** `product drift check FT-XXX` called an LLM internally.

**Amended decision:** `product drift diff FT-XXX` assembles the drift-check input
(git diff + governing ADR context) and writes it to stdout. No LLM call.

#### New command: `product drift diff`

```bash
product drift diff FT-001               # drift input for one feature
product drift diff FT-001 --format json # machine-readable
product drift diff --all-complete       # all complete features
product drift diff --changed            # features touched by recent commits
```

Output is a markdown document:

```markdown
# Drift Analysis Input: FT-001 — Cluster Foundation

## Instructions

You are checking whether recent code changes contradict the governing
architectural decisions for this feature.

Check for these drift types only:
- D001: Decision not implemented — ADR mandates X, no code implements X
- D002: Decision overridden — code does Y where ADR says do X
- D003: Partial implementation — some aspects implemented, some not
- D004: Undocumented implementation — code does X with no ADR governing why

Output format: one JSON object per line with fields: code, severity,
description, file, adr. Nothing else.

## Implementation Anchor

Feature: FT-001
Completion tag: product/FT-001/complete (2026-04-11T09:14:22Z)
Implementation files: 12 files across src/consensus/, src/storage/

## Changes Since Completion

[git diff output — bounded to implementation files since completion tag]

## Governing ADRs

[depth-2 context bundle — ADRs governing this feature]
```

#### What `product drift check` becomes

`product drift check FT-XXX` is retained as a command but now only does what it
can do structurally:

1. Confirms the completion tag exists — W020 if not
2. Checks if any implementation files have changed since the tag — reports the
   file list and change counts
3. Exits 0 if no changes, exits 2 (warning) if changes detected

```
product drift check FT-001

  Implementation anchor: product/FT-001/complete (2026-04-11T09:14:22Z)

  Changes since completion:
    src/consensus/raft.rs      +14/-3  (2026-04-14)
    src/consensus/leader.rs    +8/-1   (2026-04-14)

  ⚠ 2 implementation files changed since completion.
  Run: product drift diff FT-001 | your-llm "check for drift"
```

`product drift check` tells you *whether* drift is possible. `product drift diff`
gives an LLM what it needs to determine *what* drifted.

#### `product.toml` — removed config

```toml
# REMOVED — Product no longer calls an LLM for drift detection
[drift]
# source-roots and ignore remain — used by drift diff for file discovery
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
# max-files-per-adr removed — no longer needed without LLM call
```

---

### Amendment to ADR-022: Pre-Commit ADR Review

**Previous decision:** `product adr review --staged` ran structural checks plus an
LLM review (~3 seconds, single call).

**Amended decision:** `product adr review --staged` runs structural checks only.
The LLM portion is removed. Structural checks are instant, deterministic, and
sufficient for a pre-commit hook.

#### What remains

**Structural checks (instant, no LLM):**
- All five required sections present
- `status` field set and valid
- At least one feature linked
- At least one TC linked
- Evidence blocks present on any `⟦Γ:Invariants⟧` blocks

These checks remain. They catch the common authoring mistakes before a commit lands.

#### What is removed

**LLM checks (removed):**
- Internal consistency: does rationale support the decision?
- Contradiction scan: compare against linked ADRs' decisions
- Missing test suggestion: what TCs are obviously absent?

These move to `product gap bundle` (for gap analysis) and `product adr check-conflicts`
(for contradiction detection, which becomes structural after the next amendment below).

The pre-commit hook remains. It just runs faster.

---

### Amendment: `product adr check-conflicts`

**Previous decision:** `product adr check-conflicts ADR-XXX` called an LLM to detect
G005-class logical contradictions between ADRs.

**Amended decision:** `product adr conflict-bundle ADR-XXX` assembles the conflict-check
input and writes it to stdout. `product adr check-conflicts` runs structural checks only.

#### Structural conflict checks (no LLM)

```bash
product adr check-conflicts ADR-031
```

Checks:
- Does the new ADR's `supersedes` field form a valid chain? (E004 if cycle)
- Does any existing ADR have `superseded-by` pointing to this ADR but ADR doesn't
  have `supersedes` pointing back? (E-class inconsistency)
- Do domain declarations overlap with ADRs that have `scope: cross-cutting` in the
  same domains without an explicit acknowledgement?
- Is the `scope` field consistent with how many features link this ADR?

Fast, deterministic, always correct.

#### `product adr conflict-bundle` — LLM input

```bash
product adr conflict-bundle ADR-031     # conflict-check input for this ADR
```

Produces the new ADR plus all potentially conflicting ADRs (cross-cutting + same
domains + top-N by centrality) formatted as an LLM prompt:

```markdown
# ADR Conflict Check Input: ADR-031 — Content Hash for Tamper Detection

## Instructions

Check whether the proposed ADR logically contradicts any existing ADR.
For each contradiction found, output a JSON object with: code (always "G005"),
severity, description, conflicting-adr.

Only report genuine logical contradictions — not stylistic differences,
not overlapping concerns that are compatible, not supersession relationships.

## Proposed ADR

[full content of ADR-031]

## Existing ADRs to Check Against

[cross-cutting ADRs + same-domain ADRs + top-5 by centrality]
```

The `conflict-acknowledgements` front-matter field and the `product adr accept`
flow remain — they record the outcome of the conflict check, whether the check
was done by the user with an LLM or by review.

---

## Prompt Files

All prompt templates that previously drove internal LLM calls are retained as
resources in `benchmarks/prompts/` for users to pass to their LLM:

```
benchmarks/prompts/
  author-v1.md              # authoring (unchanged)
  implement-v1.md           # implementation context (unchanged)
  gap-analysis-v1.md        # NEW — was internal to product gap check
  drift-analysis-v1.md      # NEW — was internal to product drift check
  conflict-check-v1.md      # NEW — was internal to adr check-conflicts
```

`product prompts list` shows all five. `product prompts get gap-analysis` prints the
prompt. The user pipes `product gap bundle ADR-002` through their LLM using the
prompt as a system prompt.

---

## Updated `product.toml`

```toml
# BEFORE — had LLM config sections
[gap-analysis]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-findings-per-adr = 10
severity-threshold = "medium"

# AFTER — removed entirely
# (prompt lives in benchmarks/prompts/gap-analysis-v1.md)
# (model choice is the user's concern)
```

```toml
# BEFORE
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
max-files-per-adr = 20   # was used to cap LLM context

# AFTER
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
# max-files-per-adr removed — no LLM context to cap
```

---

## Complete LLM Call Inventory After This Change

| Location | Status | Notes |
|---|---|---|
| `product gap check` | **Removed** | Structural checks only |
| `product drift check` | **Removed** | Structural file-change detection only |
| `product adr review --staged` | **Removed** | Structural checks only |
| `product adr check-conflicts` | **Removed** | Structural consistency checks only |
| LLM benchmark (`benchmarks/`) | **Remains** | Not a product feature — self-validation |
| `product gap bundle` | **New** | Produces LLM input, no LLM call |
| `product drift diff` | **New** | Produces LLM input, no LLM call |
| `product adr conflict-bundle` | **New** | Produces LLM input, no LLM call |

Product makes zero LLM calls in production use. All semantic analysis is delegated
to the user's toolchain via the `*-bundle` and `*-diff` output commands.

---

## New Command Summary

```bash
# Unified verify pipeline
product verify                          # run all 6 stages
product verify --phase 1                # scope to phase 1
product verify FT-001                   # per-feature (existing behaviour)
product verify --ci                     # JSON output for CI

# LLM-ready outputs (no LLM call inside Product)
product gap bundle ADR-002              # gap-check input → stdout
product gap bundle --all                # all ADRs
product gap bundle --changed            # ADRs changed since last run
product drift diff FT-001               # drift-check input → stdout
product drift diff --all-complete       # all complete features
product drift diff --changed            # features affected by recent commits
product adr conflict-bundle ADR-031     # conflict-check input → stdout

# Structural-only (instant, deterministic)
product gap check                       # structural gaps only (G002, G003, G008)
product drift check FT-001              # did files change since tag?
product adr check-conflicts ADR-031     # structural consistency only
product adr review --staged             # structural checks only (pre-commit hook)
```

---

## Session Tests

```
# Verify pipeline
ST-110  verify-all-pass-clean-repo
ST-111  verify-fails-on-e-class-graph-error
ST-112  verify-warns-on-w-class-only
ST-113  verify-fails-on-failing-tc
ST-114  verify-skips-locked-phase-features
ST-115  verify-phase-scope-flag
ST-116  verify-ci-json-output
ST-117  verify-feature-scope-unchanged
ST-118  verify-log-integrity-stage-1
ST-119  verify-metrics-threshold-stage-4

# LLM boundary — gap bundle
ST-120  gap-bundle-outputs-context-and-instructions
ST-121  gap-bundle-changed-scopes-correctly
ST-122  gap-bundle-all-includes-all-adrs
ST-123  gap-check-structural-only-no-llm-call
ST-124  gap-check-g002-invariant-no-tc
ST-125  gap-check-g003-no-rejected-alternatives

# LLM boundary — drift diff
ST-126  drift-diff-outputs-diff-and-governing-adrs
ST-127  drift-diff-no-tag-warns-w020
ST-128  drift-diff-no-changes-empty-diff-section
ST-129  drift-check-structural-reports-file-changes
ST-130  drift-check-no-changes-exits-0

# LLM boundary — conflict bundle
ST-131  conflict-bundle-includes-related-adrs
ST-132  adr-check-conflicts-structural-only
```

---

## Invariants

- Product makes zero LLM API calls during `product verify`.
- `product gap check`, `product drift check`, and `product adr review --staged`
  complete in under one second on any repository of realistic size.
- `product gap bundle`, `product drift diff`, and `product adr conflict-bundle`
  produce deterministic output given the same inputs — the output is a function
  of the graph state and git history, not of any LLM.
- The prompt files in `benchmarks/prompts/` are versioned but are not executed by
  Product. They are resources. `product prompts get gap-analysis` prints the content;
  what the user does with it is their concern.
