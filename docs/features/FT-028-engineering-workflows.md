---
id: FT-028
title: Engineering Workflows
phase: 5
status: complete
depends-on: []
adrs:
- ADR-023
- ADR-024
- ADR-035
tests:
- TC-121
- TC-122
- TC-123
- TC-124
- TC-125
- TC-126
- TC-127
- TC-128
- TC-129
- TC-130
- TC-131
domains:
- api
- observability
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

### Drift Detection

`product drift` checks whether the codebase matches what the ADRs decided. The LLM receives the ADR's context bundle plus the source files most likely to implement it (resolved via configurable path patterns in `product.toml`).

```toml
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/"]
```

Drift codes:

| Code | Severity | Description |
|---|---|---|
| D001 | high | Decision not implemented — ADR says X, no code implements X |
| D002 | high | Decision overridden — code does Y, ADR says do X |
| D003 | medium | Partial implementation — some aspects of the decision implemented |
| D004 | low | Implementation ahead of spec — code does X but no ADR documents why |

Drift findings follow the same baseline/suppression model as gap findings (`drift.json`). `product drift scan src/consensus/` is the reverse direction — given source code, identify which ADRs govern it. Useful for onboarding and code review.

### Fitness Functions

`product metrics record` snapshots the current repository health into `metrics.jsonl` (one JSON line per run, committed to the repo):

```json
{"date":"2026-04-11","spec_coverage":0.87,"test_coverage":0.72,"exit_criteria_coverage":0.61,"phi":0.68,"gap_density":0.4,"gap_resolution_rate":0.75,"centrality_stability":0.02}
```

Thresholds declared in `product.toml` are checked by `product metrics threshold` in CI — this is the architectural fitness function gate. A declining `phi` below 0.70 fails CI just as a broken link does.

`product metrics trend` renders an ASCII chart to terminal for quick visual inspection.

### Pre-Commit Review

`product install-hooks` installs a pre-commit hook that runs `product adr review --staged` before every commit. The hook is advisory — it prints findings but does not block the commit. The CI gap analysis gate is the enforcement point; pre-commit is the fast-feedback loop.

The review checks locally (no LLM, instant):
- Required sections present
- At least one linked feature and one linked TC
- Status field is set
- Evidence blocks present on formal blocks

Then a single LLM call checks:
- Internal consistency of rationale
- Contradiction with linked ADRs
- Obvious missing tests given the claims made

---

---

## Description

Engineering Workflows covers three engineering feedback tools: drift detection (`product drift check`) verifies that the codebase matches what the ADRs decided by giving an LLM the ADR context bundle alongside the relevant source files (ADR-023); fitness functions (`product metrics record`, `product metrics threshold`, `product metrics trend`) track repository health metrics over time and gate CI on trend thresholds (ADR-024); and pre-commit review (`product install-hooks`, `product adr review --staged`) provides fast structural and LLM-assisted feedback on ADR drafts before they are committed (ADR-022). Together these close the spec-vs-implementation feedback loop that gap analysis (specification completeness) and graph check (structural validity) leave open.

## Functional Specification

### Inputs

- **`product drift check [ADR-XXX | FT-XXX] [--files PATH...] [--phase N]`**: artifact to check; optional explicit source file list overrides pattern-based discovery; `--phase N` checks all features in the phase
- **`product drift scan PATH`**: source file path to identify governing ADRs
- **`product.toml` `[drift]`**: `source-roots`, `ignore`, `max-files-per-adr` for pattern-based file discovery
- **`product metrics record`**: no arguments; reads current graph state and appends a snapshot to `metrics.jsonl`
- **`product metrics threshold`**: reads `[metrics.thresholds]` from `product.toml` and checks the latest `metrics.jsonl` snapshot
- **`product metrics trend [--metric NAME] [--last Nd]`**: metric name and lookback window
- **`product install-hooks`**: no arguments; writes `.git/hooks/pre-commit`
- **`product adr review --staged`**: reads staged ADR filenames from `git diff --cached --name-only`
- **`drift.json`**: baseline file for drift finding suppression (same lifecycle as `gaps.json`)

### Outputs

- **`product drift check`**: drift findings with codes D001–D004, severity, description, and suggested action; `drift.json` baseline checked for suppressions; exit 0 (clean or all suppressed), 1 (new unsuppressed drift), 2 (model error or warnings only)
- **`product drift scan PATH`**: list of governing ADRs ranked by relevance for the given source file
- **`metrics.jsonl` append**: one JSON line per `product metrics record` invocation with all tracked metrics; committed to the repository
- **`product metrics threshold`**: exit 0 (all thresholds met), 1 (threshold breach at `error` severity), 2 (threshold breach at `warning` severity only)
- **`product metrics trend`**: ASCII sparkline chart to terminal for the requested metric and window
- **`product adr review --staged` output**: rustc-style diagnostic messages for structural findings and LLM consistency findings; always exits 0 (advisory)

### State

- **`drift.json`**: committed to the repository. Records suppressions (gap ID, reason, suppressing commit, timestamp) and resolved findings. Suppressions created by one developer are respected by all CI runs.
- **`metrics.jsonl`**: committed to the repository. Grows by one line per merge to main (one `product metrics record` invocation). The full metric history is available for trend computation.
- **Implementation commits** in feature front-matter (per ADR-035): when `product verify` transitions a feature to `complete`, it records the implementation commit SHAs. Drift detection uses `git diff-tree` on these commits to determine exactly which files were changed during implementation.

### Behaviour

1. **Drift check**: Product resolves source files for an ADR via a priority order: (1) implementation commits on linked features (`git diff-tree`), (2) explicit `source-files` in ADR front-matter, (3) pattern-based discovery in `source-roots`. The LLM receives the ADR depth-2 context bundle plus resolved source files and checks for four drift types. High-severity findings (D001, D002) exit 1; medium (D003) and low (D004) findings exit 2. D004 ("undocumented implementation") is a reminder to write the ADR, not a failure.
2. **`product drift scan PATH`**: sends the source file content to the LLM with the full ADR list and asks which ADRs are relevant. Returns ADRs ranked by relevance.
3. **Fitness functions**: `product metrics record` computes all metrics from the current graph state and appends a JSON snapshot to `metrics.jsonl`. `product metrics threshold` reads the latest snapshot and checks each metric against the configured threshold. Bundle size thresholds (p95 of depth-1-adrs, tokens, domains) are recomputed from all available `bundle_measure` entries in `metrics.jsonl`.
4. **Pre-commit review**: the installed hook calls `product adr review --staged` only for staged ADR files. Structural checks run locally (instant, no LLM). LLM consistency review runs as a single call (~3 seconds). All output is advisory; the commit proceeds regardless.
5. **ADR supersession integration**: when `product adr status ADR-XXX superseded` is run, drift detection uses the transitive file sets of linked features to scope the drift check, ensuring the supersession does not silently invalidate existing implementations.

### Invariants

- `drift.json` suppressions are referenced by deterministic ID (`DRIFT-{ADR_ID}-{CODE}-{HASH}`). The same logical drift finding detected on two runs produces the same ID — suppressions remain stable across runs.
- `metrics.jsonl` is append-only. `product metrics record` never modifies existing lines. Merge conflicts are resolved by keeping both lines.
- `product adr review --staged` always exits 0. It is advisory.
- Model errors in drift check exit 2, not 1 — a transient LLM failure never causes a false drift failure in CI.
- The `source-files` field in ADR front-matter is deprecated (ADR-035). When `implementation-commits` are present on linked features, `source-files` is ignored by drift detection.

### Error handling

- **Model error during drift check**: exits 2 with a warning on stderr. Structural and configuration errors continue to exit 1.
- **No source files resolved**: exits 2 with a warning "no source files found for ADR-XXX; drift check skipped". Not an error — the ADR may not yet have an implementation.
- **`metrics.jsonl` write failure**: exits 1 with `ProductError::FileWrite`. The snapshot is not partially written (atomic write per ADR-015).
- **Threshold configuration error** (unrecognised key in `[metrics.thresholds]`): exits 1 with a configuration error before any threshold check runs.

### Boundaries

- Drift detection provides findings and suggested actions. It never automatically modifies source code or ADRs in response to drift findings. The developer decides whether to update the spec or fix the code.
- The pre-commit review is advisory. The CI gap analysis gate (`product gap check --changed`) is the enforcement point for specification quality.
- Fitness functions track repository health metrics — they do not track application runtime metrics (latency, error rates, etc.).
- `product drift scan` identifies governing ADRs for a source file but does not auto-link them to the file or create annotations.

## Out of scope

- Automatic code correction in response to drift findings
- Application runtime observability (latency, error rates, resource usage)
- Drift detection for non-ADR documents (feature files, TC files)
- Continuous background drift monitoring (invoked explicitly or in CI, not as a daemon)
- Metrics dashboards or external visualization (ASCII sparklines in terminal are the supported output)
