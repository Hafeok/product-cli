## Overview

Gap analysis is a continuous, LLM-driven specification review that identifies incompleteness, inconsistency, and missing coverage in your repository's ADRs. It checks for seven defined gap types — from missing test coverage to architectural contradictions — and produces structured JSON findings that integrate into CI. Findings are tracked over time through a `gaps.json` baseline file, supporting suppression, resolution, and audit trails. The goal is to catch specification gaps when they are introduced, not when they manifest as bugs.

## Tutorial

### Your first gap analysis run

Start by running gap analysis against a single ADR:

```bash
product gap check ADR-001
```

Product assembles a depth-2 context bundle for ADR-001 (the ADR, its linked features, their test criteria, and neighbouring ADRs), sends it to the configured LLM, and checks for seven specific gap types. The output is structured JSON on stdout:

```json
{
  "adr": "ADR-001",
  "run_date": "2026-04-14T10:00:00Z",
  "product_version": "0.1.0",
  "findings": [
    {
      "id": "GAP-ADR001-G001-b2c4",
      "code": "G001",
      "severity": "high",
      "description": "The claim 'graph rebuild completes in under 50ms for 100 artifacts' has no linked TC exercising it.",
      "affected_artifacts": ["ADR-001"],
      "suggested_action": "Add a scenario or invariant TC that benchmarks graph rebuild time.",
      "suppressed": false
    }
  ],
  "summary": { "high": 1, "medium": 0, "low": 0, "suppressed": 0 }
}
```

Exit code 1 means new, unsuppressed gaps were found. Exit code 0 means clean.

### Understanding the findings

Each finding includes:

- **id** — a deterministic identifier (e.g. `GAP-ADR001-G001-b2c4`) that remains stable across runs
- **code** — which of the seven gap types was triggered (e.g. G001 = missing test coverage)
- **severity** — `high`, `medium`, or `low`
- **suggested_action** — a concrete next step to resolve the gap

### Suppressing a known gap

If a gap is expected or deliberately deferred, suppress it so it no longer fails CI:

```bash
product gap suppress GAP-ADR001-G001-b2c4 --reason "Benchmark TC deferred to phase 3"
```

This writes a suppression entry to `gaps.json` at the repository root. Commit `gaps.json` so the suppression is shared with your team. On subsequent runs, the gap appears with `"suppressed": true` and does not cause exit code 1.

### Running in CI mode

In a CI pipeline, use `--changed` to scope analysis to ADRs affected by the current commit:

```bash
product gap check --changed
```

This identifies changed ADR files via `git diff`, expands to include 1-hop graph neighbours, and analyses only that subset. CI time stays proportional to change scope.

## How-to Guide

### Analyse a specific ADR

```bash
product gap check ADR-002
```

Analyses ADR-002 using its full depth-2 context bundle. Reports all detected gaps as JSON on stdout.

### Analyse all changed ADRs in a PR

```bash
product gap check --changed
```

1. Detects which ADR files changed in the current commit (`git diff --name-only HEAD~1`).
2. Expands the set to include 1-hop ADR neighbours (ADRs that share a feature with any changed ADR).
3. Analyses each ADR in the expanded set.

### Suppress a gap

1. Note the gap ID from the analysis output (e.g. `GAP-ADR002-G001-a3f9`).
2. Run:
   ```bash
   product gap suppress GAP-ADR002-G001-a3f9 --reason "Split-brain chaos test deferred to phase 2"
   ```
3. Commit the updated `gaps.json`.

The suppression records the gap ID, reason, current commit hash, and timestamp.

### Re-confirm suppressions after a prompt version upgrade

When `prompt-version` is incremented in `product.toml`, existing suppressions are flagged with warnings. Review each warning and re-confirm:

```bash
product gap suppress --re-confirm
```

Or remove suppressions that are no longer valid by editing `gaps.json` directly.

### Verify that a fix resolved a gap

1. Fix the underlying issue (e.g. add the missing TC).
2. Run gap analysis again:
   ```bash
   product gap check ADR-002
   ```
3. If the gap is no longer detected, it is automatically moved to the `resolved` list in `gaps.json`.

### Integrate into a CI pipeline

Add to your CI configuration:

```yaml
- name: Gap analysis
  run: product gap check --changed
```

Exit codes:
- **0** — no new unsuppressed gaps
- **1** — new unsuppressed gaps found (fails the build)
- **2** — analysis warning (model error, network failure) — does not fail the build

### Separate findings from errors in scripts

Gap findings go to stdout; errors and warnings go to stderr. To capture only findings:

```bash
product gap check ADR-001 > findings.json 2> errors.log
```

`findings.json` is always valid JSON (or empty on exit code 2).

## Reference

### Commands

| Command | Description |
|---|---|
| `product gap check <ADR-ID>` | Analyse a single ADR |
| `product gap check --changed` | Analyse ADRs changed in the current commit plus 1-hop neighbours |
| `product gap suppress <GAP-ID> --reason "<text>"` | Suppress a gap in `gaps.json` |
| `product gap suppress --re-confirm` | Re-confirm suppressions after a prompt version upgrade |

### Exit codes

| Code | Meaning |
|---|---|
| 0 | No new unsuppressed gaps |
| 1 | New unsuppressed gaps found |
| 2 | Analysis warning (model error, invalid response) |

### Gap types

| Code | Severity | Description |
|---|---|---|
| G001 | high | Missing test coverage — ADR makes a testable claim with no linked TC |
| G002 | high | Untested formal invariant — `⟦Γ:Invariants⟧` block with no scenario or chaos TC |
| G003 | medium | Missing rejected alternatives section |
| G004 | medium | Undocumented constraint referenced in rationale |
| G005 | high | Architectural contradiction with a linked ADR |
| G006 | medium | Feature coverage gap — feature aspect not addressed by any linked ADR |
| G007 | low | Stale rationale referencing a superseded ADR |

### Finding JSON schema

Every finding in the output contains these required fields:

| Field | Type | Description |
|---|---|---|
| `id` | string | Deterministic gap ID: `GAP-{ADR_ID}-{CODE}-{HASH}` |
| `code` | string | Gap type code (G001–G007) |
| `severity` | string | `high`, `medium`, or `low` |
| `description` | string | One-sentence description of the gap |
| `affected_artifacts` | string[] | IDs of artifacts involved |
| `suggested_action` | string | Recommended fix |
| `suppressed` | boolean | Whether the gap is suppressed in `gaps.json` |

The `evidence` field is required for G005 findings, quoting the conflicting claims from each ADR.

### Gap ID format

```
GAP-{ADR_ID}-{CODE}-{SHORT_HASH}
```

The short hash is derived from `sha256(adr_id + gap_code + sorted(affected_artifact_ids) + description)[0:4]`. The same logical gap produces the same ID across runs. All IDs match the pattern `GAP-[A-Z]+-[A-Z0-9]+-[A-Z0-9]{4,8}`.

### Configuration (`product.toml`)

```toml
[gap-analysis]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-findings-per-adr = 10
severity-threshold = "medium"
```

| Key | Default | Description |
|---|---|---|
| `prompt-version` | `"1"` | Version of the gap analysis prompt (stored at `benchmarks/prompts/gap-analysis-v{N}.md`) |
| `model` | `"claude-sonnet-4-6"` | LLM model used for analysis |
| `max-findings-per-adr` | `10` | Maximum findings reported per ADR |
| `severity-threshold` | `"medium"` | Gaps below this severity are informational only |

### Baseline file (`gaps.json`)

Located at the repository root. Contains two lists:

- **suppressions** — gaps deliberately accepted, with reason, commit hash, and timestamp
- **resolved** — gaps previously tracked that are no longer detected

If `gaps.json` does not exist, all findings are treated as new.

### Prompt file

The analysis prompt is stored at `benchmarks/prompts/gap-analysis-v{N}.md` where `{N}` matches the `prompt-version` in `product.toml`. The prompt is fixed and versioned — findings from different prompt versions are not comparable.

## Explanation

### Why a fixed set of gap types?

An open-ended "find any problems" prompt produces unbounded, incomparable findings across runs. By enumerating exactly seven gap types, gap analysis becomes a bounded, repeatable check — closer to a linter than a code review. Each gap type has a clear trigger condition, making findings actionable and suppressions meaningful. This constraint is essential for CI reliability (ADR-019).

### How determinism works in a non-deterministic system

LLM output is inherently non-deterministic. Three mechanisms stabilise gap analysis for CI use:

1. **Temperature=0** reduces but does not eliminate variation.
2. **Structured JSON output** — the model is constrained to a specific schema. Findings that cannot be parsed are discarded, not propagated.
3. **Run-twice intersection** — for high-severity findings (G001, G002, G005), the analysis runs twice. Only findings present in both runs are reported. This eliminates hallucinated gaps at the cost of occasionally missing real ones. For CI, false negatives are preferable to false positives.

Medium and low severity findings use single-run analysis only, since false positives at these levels do not fail CI.

### Why `--changed` expands to 1-hop neighbours

A change to ADR-002 might introduce a contradiction with ADR-005 if they share a feature. Analysing only ADR-002 would miss this G005. The 1-hop expansion through shared features is the minimum scope that catches cross-ADR contradictions introduced by a change, while keeping the analysis set bounded and CI time proportional to change scope.

### Why model errors are warnings, not failures

A transient API error or network timeout should never fail a CI build. Gap analysis exits 2 (warning) on model errors, reserving exit code 1 exclusively for new unsuppressed gaps. The operator can re-run manually, or the next commit triggers another analysis. This asymmetry keeps CI stable even when the model provider has intermittent issues.

### Relationship to the knowledge graph

Gap analysis uses the same depth-2 context bundles that implementation agents receive (`product context ADR-XXX --depth 2`). This means it validates specification completeness from the agent's perspective — if an agent cannot find the information it needs in the bundle, gap analysis flags that as a gap. This alignment between analysis scope and implementation scope is by design (ADR-019).

### The `gaps.json` suppression model

The suppression workflow follows the `cargo audit` pattern: detect, review, suppress known issues with a documented reason, fail on new ones. `gaps.json` is committed to the repository, making suppressions a shared team decision with an audit trail. This is preferable to storing findings in ADR front-matter, which would create noise in git history and contaminate specification content with tooling metadata (ADR-019).
