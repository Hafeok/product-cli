## Overview

Engineering Workflows bundles three capabilities that keep specifications and code aligned over time: **drift detection** verifies that source code matches what ADRs decided, **fitness functions** track architectural health metrics across merges, and **pre-commit review** gives fast local feedback before code reaches CI. Together they close the feedback loop between the documentation graph and the implementation.

## Tutorial

### Detect your first spec drift

Drift detection compares ADR decisions against your actual source code using an LLM. Start by checking a single ADR:

```bash
product drift check ADR-002
```

Product resolves the source files associated with ADR-002 (via pattern matching or explicit `source-files` in the ADR's front-matter), assembles a context bundle, and asks the LLM to identify divergences. Output includes drift findings with codes like D001 (not implemented) or D002 (overridden).

### Record your first metrics snapshot

After a merge to main, record the current state of your repository's architectural health:

```bash
product metrics record
```

This appends a single JSON line to `metrics.jsonl` containing coverage ratios, formal block coverage (`phi`), gap density, and other tracked values.

### Check thresholds

Run the threshold gate to see whether your repository meets configured standards:

```bash
product metrics threshold
```

If all thresholds pass, the command exits 0. If a warning-severity threshold is breached, it exits 2. If an error-severity threshold is breached, it exits 1.

### Visualize trends

See how a metric has changed over time:

```bash
product metrics trend --metric phi --last 30d
```

This renders an ASCII sparkline in your terminal showing the trajectory, current value, deltas, and threshold line.

### Install pre-commit hooks

Set up the advisory pre-commit hook for local ADR review:

```bash
product install-hooks
```

From now on, every `git commit` runs `product adr review --staged`, printing findings without blocking the commit.

## How-to Guide

### Check whether code matches a specific ADR

1. Run `product drift check ADR-XXX`.
2. Review the findings. Each finding has a drift code (D001-D004) and severity.
3. If a finding is intentional (e.g., partial implementation planned for a later phase), suppress it in `drift.json`.

### Override source file discovery for an ADR

If pattern-based discovery misses relevant files, specify them explicitly:

**Option A — CLI flag:**

```bash
product drift check ADR-002 --files src/consensus/raft.rs src/consensus/leader.rs
```

**Option B — ADR front-matter (recommended for permanent associations):**

```yaml
---
id: ADR-002
source-files:
  - src/consensus/raft.rs
  - src/consensus/leader.rs
---
```

Explicit `source-files` in front-matter always override pattern-based discovery.

### Find which ADRs govern a source file

1. Run `product drift scan src/consensus/raft.rs`.
2. The output lists ADRs ranked by relevance to that file.
3. Use this during code review or onboarding to understand the decisions behind unfamiliar code.

### Suppress a known drift finding

1. Run `product drift check` and note the finding ID (e.g., `DRIFT-ADR002-D003-f4a1`).
2. Add a suppression entry to `drift.json`:

```json
{
  "schema-version": "1",
  "suppressions": [
    {
      "id": "DRIFT-ADR002-D003-f4a1",
      "reason": "Partial implementation is intentional — full storage layer in phase 2",
      "suppressed_by": "git:abc123",
      "suppressed_at": "2026-04-11T09:00:00Z"
    }
  ]
}
```

3. Subsequent drift checks treat that finding as suppressed and exit cleanly.

### Add metrics threshold checks to CI

1. Configure thresholds in `product.toml`:

```toml
[metrics.thresholds]
spec_coverage = { min = 0.90, severity = "error" }
test_coverage = { min = 0.80, severity = "error" }
phi = { min = 0.70, severity = "warning" }
drift_density = { max = 0.20, severity = "warning" }
```

2. Add `product metrics record` to your merge-to-main pipeline.
3. Add `product metrics threshold` as a CI gate step. It exits non-zero on breaches.

### View a summary of all metrics

Run `product metrics trend` with no flags to see a table of all metrics with current value, 7-day delta, and trend arrow.

## Reference

### `product drift check`

```
product drift check [ADR-ID] [OPTIONS]
```

Checks whether source code matches ADR decisions using LLM analysis.

| Flag | Description |
|---|---|
| `ADR-ID` | Optional. Check a specific ADR. Without it, checks all ADRs. |
| `--files <PATH>...` | Override source file discovery with explicit file paths. |

**Exit codes:** 0 = no drift, non-zero = drift findings detected (follows ADR-009 exit code model). Suppressed findings do not cause non-zero exit.

### `product drift scan`

```
product drift scan <PATH>
```

Given a source file path, identifies which ADRs govern it, ranked by relevance.

| Argument | Description |
|---|---|
| `<PATH>` | Source file to analyze (e.g., `src/consensus/raft.rs`). |

### Drift codes

| Code | Severity | Meaning |
|---|---|---|
| D001 | high | Decision not implemented -- ADR mandates X, no code implements it |
| D002 | high | Decision overridden -- code does Y where ADR says do X |
| D003 | medium | Partial implementation -- some aspects implemented, others missing |
| D004 | low | Undocumented implementation -- code does X with no governing ADR |

### Drift configuration (`product.toml`)

```toml
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
max-files-per-adr = 20
```

| Key | Description |
|---|---|
| `source-roots` | Directories to search for source files associated with ADRs. |
| `ignore` | Directories excluded from source file discovery. |
| `max-files-per-adr` | Maximum files included per ADR to bound context bundle size. |

### `drift.json`

Baseline file for suppressions. Structure mirrors `gaps.json`. Suppression IDs follow the pattern `DRIFT-{ADR_ID}-{CODE}-{HASH}`.

### `product metrics record`

```
product metrics record
```

Appends a JSON snapshot to `metrics.jsonl`. No flags. Run once per merge to main.

**Output format** (one JSON object per line in `metrics.jsonl`):

```json
{"date":"2026-04-11T09:00:00Z","commit":"abc123","spec_coverage":0.87,"test_coverage":0.72,"exit_criteria_coverage":0.61,"phi":0.68,"gap_density":0.03,"gap_resolution_rate":0.75,"drift_density":0.10,"centrality_stability":0.02,"implementation_velocity":2}
```

### Tracked metrics

| Metric | Range | Computation | Good direction |
|---|---|---|---|
| `spec_coverage` | [0, 1] | Features with >= 1 linked ADR / total features | Up |
| `test_coverage` | [0, 1] | Features with >= 1 linked TC / total features | Up |
| `exit_criteria_coverage` | [0, 1] | Features with exit-criteria TC / total features | Up |
| `phi` | [0, 1] | Mean formal block coverage across invariant+chaos TCs | Up |
| `gap_density` | [0, 1] | New gaps in last 7d / total ADRs | Down |
| `gap_resolution_rate` | [0, 1] | Gaps resolved / gaps opened, rolling 30d | Down |
| `drift_density` | [0, 1] | Unresolved drift findings / total ADRs | Down |
| `centrality_stability` | variance | Variance in top-5 ADR centrality ranks, week-over-week | Down |
| `implementation_velocity` | count | Features moved to `complete` in last 7d | Tracked only |

### `product metrics threshold`

```
product metrics threshold
```

Checks current metric values against thresholds in `product.toml`.

| Exit code | Meaning |
|---|---|
| 0 | All thresholds met |
| 1 | Error-severity threshold breached |
| 2 | Warning-severity threshold breached |

### Threshold configuration (`product.toml`)

```toml
[metrics.thresholds]
spec_coverage = { min = 0.90, severity = "error" }
test_coverage = { min = 0.80, severity = "error" }
exit_criteria_coverage = { min = 0.60, severity = "warning" }
phi = { min = 0.70, severity = "warning" }
gap_resolution_rate = { min = 0.50, severity = "warning" }
drift_density = { max = 0.20, severity = "warning" }
```

Use `min` for metrics where higher is better, `max` for metrics where lower is better. Severity is either `"error"` (exit 1) or `"warning"` (exit 2).

### `product metrics trend`

```
product metrics trend [OPTIONS]
```

| Flag | Description |
|---|---|
| `--metric <NAME>` | Show sparkline for a single metric. Without it, shows summary table. |
| `--last <DURATION>` | Time window (e.g., `30d`, `7d`). |

Output includes current value, 7-day delta, 30-day delta, and trend arrow.

### `product install-hooks`

```
product install-hooks
```

Installs a Git pre-commit hook that runs `product adr review --staged`. The hook is **advisory** -- it prints findings but does not block the commit.

### Pre-commit review checks

**Local checks (no LLM, instant):**
- Required ADR sections present
- At least one linked feature and one linked TC
- Status field is set
- Evidence blocks present on formal blocks

**LLM check (single call):**
- Internal consistency of rationale
- Contradiction with linked ADRs
- Missing tests given the claims made

## Explanation

### Why drift detection is separate from gap analysis

Gap analysis (ADR-019) operates entirely within the documentation graph -- it checks whether ADRs are internally consistent and well-covered by test criteria. Drift detection crosses the docs/code boundary, comparing ADR decisions against actual source files. This fundamental difference in scope justifies a separate command (`product drift` vs `product gap`), separate finding codes (D001-D004 vs gap codes), and a separate baseline file (`drift.json` vs `gaps.json`). See ADR-023 for the full rationale.

### The two directions of drift analysis

`product drift check` goes from spec to code: "does the code match what this ADR decided?" `product drift scan` goes from code to spec: "which ADRs govern this file?" The check direction is for verification; the scan direction is for discovery and onboarding. Both use LLM analysis, but they serve different workflows.

### Why metrics are stored in a committed file

`metrics.jsonl` is committed to the repository rather than stored in an external metrics service. This follows Product's repository-native design principle -- the metric history lives alongside the artifacts it measures, requires no external dependencies, and is inspectable with `git log -p metrics.jsonl`. The file is append-only (one line per `product metrics record` invocation), so merge conflicts are resolved by keeping both lines. See ADR-024 for this decision.

### Why `implementation_velocity` has no threshold

Fast velocity is not always good (quality may be declining), and slow velocity is not always bad (hard problems take time). It is tracked for observation and trend awareness, not gated. This is an intentional design choice documented in ADR-024.

### Advisory hooks vs. CI enforcement

The pre-commit hook installed by `product install-hooks` is deliberately advisory. It provides fast feedback during development but does not block commits. The CI pipeline (via `product metrics threshold` and `product gap check`) is the enforcement point. This two-tier approach avoids developer frustration from blocked commits while still maintaining quality gates where they matter.
