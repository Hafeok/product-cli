---
id: FT-044
title: Unified Verify Pipeline
phase: 5
status: planned
depends-on:
- FT-018
- FT-037
- FT-042
adrs:
- ADR-009
- ADR-013
- ADR-021
- ADR-024
- ADR-036
- ADR-040
tests:
- TC-552
- TC-553
- TC-554
- TC-555
- TC-556
- TC-557
- TC-558
- TC-559
- TC-560
- TC-561
- TC-562
domains:
- api
- error-handling
- observability
domains-acknowledged: {}
---

## Description

`product verify` with no arguments runs the full six-stage verification pipeline. This feature delivers the pipeline, the stage ordering, the exit-code semantics, the `--ci` JSON output, and the `--phase` scope flag. It is the single entry point that answers "is this repository in a good state?"

This feature implements ADR-040.

---

## Depends on

- **FT-018** — Validation and graph health. Stage 2 wraps `product graph check`; the E/W-code vocabulary is the feature's output contract.
- **FT-037** — Tag-Based Drift Detection. Stage 5 skips features whose completion tag is in a locked phase; stage 1 cross-references git tags.
- **FT-042** — Request Log Hash-Chain and Replay. Stage 1 wraps `product request log verify`.
- **FT-029** — Gap Analysis. `product gap check` (now structural-only after this feature) must remain callable independently; the pipeline does not call it.

---

## Scope of this feature

### In

1. **The `product verify` entry point with no arguments.** Runs all six stages in order, always completes (never short-circuits on error), exits with the worst result across all stages: 0 (all pass), 1 (any E-class error), 2 (warnings only).
2. **Stage 1 — Log integrity.** Wraps `product request log verify`. Errors: E015 (hash mismatch), E016 (chain break). Warnings: W021 (tag without log entry).
3. **Stage 2 — Graph structure.** Wraps `product graph check`. Errors: any E-class finding. Warnings: W-class findings.
4. **Stage 3 — Schema validation.** Compares `schema-version` in `product.toml` against the binary's supported schema version. Errors: E008. Warnings: W007.
5. **Stage 4 — Metrics thresholds.** Wraps `product metrics threshold`. Exit status by severity=error vs severity=warning thresholds in `[metrics.thresholds]`.
6. **Stage 5 — Feature TCs.** For each feature reachable from the current phase gate: skip if `status: planned`, otherwise run `product verify FT-XXX`. Features in locked phases are skipped with a note. Per-TC status recorded: pass / fail / unrunnable. Features from locked phases are listed under a "skipped" section with their phase.
7. **Stage 6 — Platform TCs.** Wraps `product verify --platform` — TCs linked to cross-cutting ADRs.
8. **Scope flags.** `--phase N` scopes stage 5 to features in that phase (and all stages still run); `FT-XXX` as the positional argument keeps existing per-feature behaviour unchanged.
9. **`--ci` flag.** Emits the stage-by-stage result as a single structured JSON document on stdout (schema below). No colour, no TTY-specific formatting. Suitable for GitHub Actions, Jenkins, CircleCI parsing.
10. **Output formatting.** Pretty mode prints one line per stage with ✓ / ⚠ / ✗ prefix, aggregated finding counts, a summary block listing exit code, failing TCs, and features needing attention (by W-code).

### Out

- **LLM-dependent checks** (gap analysis, drift detection, semantic ADR review) are deliberately not in the pipeline — see FT-045. `product verify` is strictly deterministic.
- **Parallel stage execution.** Stages run sequentially. Parallelism is a future optimisation; it is not required for correctness.
- **Stage caching.** No caching between invocations. Every `product verify` re-runs every stage from scratch.
- **The per-feature `product verify FT-XXX` command itself.** Already delivered (ADR-021, FT-037); this feature only invokes it as stage 5.

---

## Commands

```bash
product verify                      # everything — all phases, all features
product verify --phase 1            # scope stage 5 to phase 1 features only
product verify FT-001               # per-feature (unchanged behaviour)
product verify --ci                 # structured JSON output, no colour
product verify --ci --phase 1       # combined
```

---

## `--ci` Output Schema

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
        { "tc": "TC-012", "feature": "FT-004", "status": "failing" },
        { "tc": "TC-050", "feature": "FT-009", "status": "skipped", "reason": "phase-2-locked" }
    ]},
    { "stage": 6, "name": "platform-tcs",      "status": "pass",    "findings": [] }
  ]
}
```

Each stage object has: `stage` (1–6), `name`, `status` (pass | warning | fail), `findings` (array). Finding shape depends on stage: strings for diagnostic codes, objects for TC results. `passed` is true only if `exit == 0`.

---

## Output Format (pretty mode)

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

---

## Implementation notes

- **New module: `src/verify/pipeline.rs`** (or extend the existing `src/verify/` structure per ADR-029). Houses `PipelineStage`, `StageResult`, `StageStatus`, and the six stage runners. Each stage runner delegates to existing code (`commands::graph_check`, `commands::metrics::threshold`, `commands::verify::verify_feature`, `commands::request::log_verify`).
- **New command handler in `src/commands/verify.rs`.** When called with no positional argument, dispatch to `pipeline::run_all(&config, &graph, scope)`. When called with `FT-XXX`, dispatch to the existing per-feature verify. The `--ci` flag switches formatter from `pretty` to `json`.
- **Exit code unification.** `main.rs` currently maps each command's `Err` to exit 1. The pipeline returns a `PipelineResult` whose `.exit_code()` method produces 0 / 1 / 2 directly; the command handler propagates this without going through `ProductError`.
- **Stage independence.** Each stage is invoked in a `Result::catch_unwind` style wrapper (via `std::panic::catch_unwind` on the inner closure) so that a stage panic does not abort the pipeline. The panicking stage is marked `fail` with a diagnostic in `findings` and the pipeline continues.
- **Locked phase detection.** A phase is locked if the next-higher phase contains at least one feature with `status: complete` (the phase gate in ADR-034 opened the next phase). Features in phases strictly older than the current phase are skipped with reason `"phase-N-locked"`.
- **Testing.** Every TC added for this feature (TC-552..TC-562) lives under `tests/sessions/` per FT-043 conventions. Each session test composes a temp repository, drives `product verify` through a controlled state, and asserts on the JSON output + exit code. Runner config (`runner: cargo-test`, `runner-args: "tc_XXX_snake_case"`) is added at the same time the test is written.

---

## Acceptance criteria

A developer running `product verify` on a clean repository can:

1. Observe all six stages execute and the command exit 0 (TC-552).
2. Observe `product verify` exit 1 when any E-class graph error is present and see which codes triggered the failure (TC-553).
3. Observe `product verify` exit 2 when only W-class warnings are present, with the warnings listed per stage (TC-554).
4. Observe `product verify` exit 1 when any feature TC is failing, with the specific TC and feature named in the output (TC-555).
5. Observe features in a locked phase are skipped with a named reason, not executed (TC-556).
6. Run `product verify --phase 1` and observe stage 5 runs only phase 1 features (TC-557).
7. Run `product verify --ci` and parse the resulting JSON as valid, matching the documented schema (TC-558).
8. Run `product verify FT-001` and observe the existing per-feature behaviour is unchanged (TC-559).
9. Observe stage 1 detects a tampered request log and fails the pipeline with E015 or E016 (TC-560).
10. Observe stage 4 reports a metric threshold breach and exits 2 when the breach is warning-level (TC-561).
11. Observe `product verify` completes in less than 10 seconds on a realistic repository when no custom TCs are run (covered as implicit performance expectation; not a separate TC).
12. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.

See TC-562 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **Parallel stage execution** — revisit if stage-5 dominates wall time enough to justify the additional failure-mode surface area.
- **Incremental verify** — remember which stages changed since the last invocation via content hashes on the graph nodes and skip unchanged stages. Only worthwhile if wall time becomes a complaint.
- **Pipeline hooks** — allow `product.toml` to declare additional pre/post-stage shell commands. Today the escape hatch is "call the shell command yourself after `product verify`".
