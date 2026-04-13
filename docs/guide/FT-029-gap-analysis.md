The file write was denied. Here's the complete documentation for FT-029 — Gap Analysis. You can save it to `docs/guide/FT-029-gap-analysis.md`:

---

## Overview

Gap analysis is a continuous specification review process that identifies incompleteness, inconsistency, and missing test coverage in your repository's ADRs. It checks for seven defined gap types (G001–G007), produces structured JSON findings, and integrates with a `gaps.json` baseline file for suppression and resolution tracking. In CI, the `--changed` flag scopes analysis to modified ADRs and their graph neighbours, keeping cost proportional to change scope. Gap analysis is implemented by ADR-019.

## Tutorial

### Your first gap analysis run

Run gap analysis against a single ADR to see what it finds:

```bash
product gap check ADR-001
```

The output is a JSON array of gap reports written to stdout. Each report contains the ADR ID, a list of findings, and a severity summary. If unsuppressed high-severity gaps are found, the command exits with code 1.

### Understanding the output

A typical finding looks like this:

```json
{
  "id": "GAP-ADR-001-G003-a1b2",
  "code": "G003",
  "severity": "medium",
  "description": "ADR has no Rejected alternatives section",
  "affected_artifacts": ["ADR-001"],
  "suggested_action": "Add a **Rejected alternatives** section documenting considered alternatives.",
  "suppressed": false
}
```

Each finding has a deterministic ID, a gap code (G001–G007), a severity, and a suggested action.

### Suppressing a known gap

If a gap is expected or deferred, suppress it so it no longer fails CI:

```bash
product gap suppress GAP-ADR-001-G003-a1b2 --reason "deferred to phase 2"
```

This writes a suppression entry to `gaps.json`. Commit the file so the suppression is shared with your team.

### Running in CI mode

Use `--changed` to analyse only ADRs modified in the last commit, plus their 1-hop neighbours:

```bash
product gap check --changed
```

This is the recommended CI integration. It keeps analysis time bounded and proportional to the size of the change.

### Viewing a human-readable report

For a summary across all ADRs:

```bash
product gap report
```

This prints a formatted report with finding counts by severity and per-ADR details.

## How-to Guide

### Check all ADRs for gaps

```bash
product gap check
```

Analyses every ADR in the repository. Returns exit code 0 if no unsuppressed high-severity gaps are found.

### Check a specific ADR

```bash
product gap check ADR-019
```

### Check only changed ADRs (CI mode)

```bash
product gap check --changed
```

Uses `git diff --name-only HEAD~1` to identify changed ADR files, then expands the analysis set to include 1-hop graph neighbours (ADRs sharing a feature with any changed ADR).

### Get text output instead of JSON

```bash
product gap check --format text
```

Prints a human-readable listing with severity markers instead of JSON. Only ADRs with findings are shown.

### Suppress a gap finding

```bash
product gap suppress GAP-ADR-002-G001-a3f9 --reason "split-brain chaos test deferred to phase 2"
```

Records the suppression in `gaps.json` with the reason, current git commit hash, and timestamp. Commit `gaps.json` to share the suppression.

### Remove a suppression

```bash
product gap unsuppress GAP-ADR-002-G001-a3f9
```

Removes the suppression entry from `gaps.json`. The gap will be reported again on the next run.

### View gap statistics

```bash
product gap stats
```

Outputs a JSON object with total findings, unsuppressed counts by severity, suppressed count, resolved count, and number of ADRs analysed.

### Add gap analysis to CI

Add this step to your CI pipeline:

```yaml
- name: Gap analysis
  run: product gap check --changed
```

Exit codes:
- **0** — no unsuppressed high-severity gaps
- **1** — at least one unsuppressed high-severity gap found
- **2** — model call failure (warning, not a gate failure)

## Reference

### Subcommands

| Subcommand | Description |
|---|---|
| `gap check` | Run gap analysis on ADRs |
| `gap report` | Print a human-readable gap report for all ADRs |
| `gap suppress` | Suppress a gap finding in `gaps.json` |
| `gap unsuppress` | Remove a suppression from `gaps.json` |
| `gap stats` | Print gap statistics as JSON |

### `gap check`

```
product gap check [ADR_ID] [--changed] [--format <FORMAT>]
```

| Option | Type | Default | Description |
|---|---|---|---|
| `ADR_ID` | positional, optional | — | Specific ADR to check. Omit to check all. |
| `--changed` | flag | `false` | Only check ADRs modified in the last commit, expanded with 1-hop neighbours. |
| `--format` | `json` \| `text` | `json` | Output format. |

**Exit codes:**

| Code | Meaning |
|---|---|
| 0 | No unsuppressed high-severity gaps |
| 1 | At least one unsuppressed high-severity gap found |
| 2 | Model call failure (analysis incomplete) |

### `gap suppress`

```
product gap suppress <GAP_ID> --reason <REASON>
```

| Option | Type | Required | Description |
|---|---|---|---|
| `GAP_ID` | positional | yes | The full gap ID (e.g., `GAP-ADR-001-G003-a1b2`) |
| `--reason` | string | yes | Reason for suppression |

### `gap unsuppress`

```
product gap unsuppress <GAP_ID>
```

| Option | Type | Required | Description |
|---|---|---|---|
| `GAP_ID` | positional | yes | The full gap ID to unsuppress |

### `gap report`

```
product gap report
```

No arguments or flags. Always exits 0.

### `gap stats`

```
product gap stats
```

No arguments or flags. Output JSON schema:

```json
{
  "total_findings": 5,
  "unsuppressed": { "high": 1, "medium": 2, "low": 1 },
  "suppressed": 1,
  "resolved": 2,
  "adrs_analysed": 12
}
```

### Gap types

| Code | Severity | Description |
|---|---|---|
| G001 | high | ADR has testable claims with no linked TC |
| G002 | high | `⟦Γ:Invariants⟧` formal block present but no scenario or chaos TC exercises it |
| G003 | medium | ADR has no **Rejected alternatives** section |
| G004 | medium | ADR rationale references an external constraint not captured in any linked artifact |
| G005 | high | ADR makes a claim logically inconsistent with a linked ADR |
| G006 | medium | Feature has aspects not addressed by any linked ADR |
| G007 | low | ADR rationale references decisions superseded by a more recent ADR |

G001, G002, G003, G006, and G007 are structural checks that run locally. G004 and G005 are semantic checks requiring LLM analysis.

### Gap ID format

```
GAP-{ADR_ID}-{CODE}-{HASH}
```

Example: `GAP-ADR-002-G001-a3f9`

The hash is derived from `sha256(adr_id + code + sorted(affected_artifact_ids) + description)[0:4]`, making IDs deterministic for the same logical gap.

Pattern: `GAP-[A-Z]+-[A-Z0-9]+-[A-Z0-9]{4,8}`

### Output JSON schema

**Gap report** (array element):

```json
{
  "adr": "ADR-002",
  "run_date": "2026-04-11T09:00:00Z",
  "product_version": "0.1.0",
  "findings": [ ],
  "summary": { "high": 1, "medium": 0, "low": 0, "suppressed": 0 }
}
```

**Gap finding**:

```json
{
  "id": "GAP-ADR-002-G001-a3f9",
  "code": "G001",
  "severity": "high",
  "description": "...",
  "affected_artifacts": ["ADR-002"],
  "suggested_action": "...",
  "suppressed": false
}
```

All findings go to **stdout**. Errors and warnings go to **stderr**.

### Baseline file (`gaps.json`)

Located at the repository root. Committed to version control.

```json
{
  "schema-version": "1",
  "suppressions": [
    {
      "id": "GAP-ADR-002-G001-a3f9",
      "reason": "Split-brain chaos test deferred to phase 2",
      "suppressed_by": "git:abc123",
      "suppressed_at": "2026-04-11T09:00:00Z"
    }
  ],
  "resolved": [
    {
      "id": "GAP-ADR-001-G003-c4d5",
      "resolved_at": "2026-04-12T14:30:00Z",
      "resolving_commit": "git:def456"
    }
  ]
}
```

## Explanation

### Why gap analysis belongs in CI

Specification gaps compound over time. An untested claim in an ADR today becomes a misunderstood invariant, then a production bug months later. Running gap analysis continuously catches gaps when they are introduced — not when they manifest as implementation defects. The CI gate (exit code 1 on unsuppressed high-severity gaps) ensures gaps are addressed, not just reported.

### Why seven fixed gap types

An open-ended "find any problems" prompt produces unbounded, incomparable findings across runs. By enumerating exactly seven gap types, gap analysis becomes a bounded, checkable specification — more like a linter than a code review. Each gap type has a clear trigger condition and severity, making the output predictable and actionable. New gap types can be added in future prompt versions, but the set is always fixed for a given version.

### Determinism strategy for CI

LLM output is non-deterministic. Three measures stabilise gap analysis:

1. **Temperature=0** for all gap analysis calls
2. **Structured JSON output only** — findings that cannot be parsed are discarded with a warning, not propagated as failures
3. **Run twice, intersect** — for high-severity findings (G001, G002, G005), the analysis runs twice and only findings present in both runs are reported

The run-twice approach is conservative: some real gaps may require two consistent runs to surface. But a false G005 (architectural contradiction) that fails CI is highly disruptive, so conservative is correct.

### `--changed` scoping and 1-hop expansion

Full-repository analysis on every commit is prohibitively expensive. The `--changed` flag uses `git diff --name-only HEAD~1` to find changed ADRs, then expands the analysis set to 1-hop graph neighbours — ADRs that share a feature with any changed ADR. This expansion is necessary to catch G005 (contradiction): a change to ADR-002 may now contradict ADR-005 if they share a feature. Without expansion, cross-ADR contradictions would never be detected in CI.

The analysis set is bounded: `|changed_adrs| × |avg_adr_neighbours|`. For a typical repository, a PR changing 2 ADRs analyses at most ~8 ADRs.

### The suppression model

`gaps.json` follows the `cargo audit` model: audit findings, suppress known/expected issues with a reason, fail on new ones. Suppressions record the gap ID, reason, suppressing commit, and timestamp — creating an audit trail of deliberate decisions to accept known gaps. Because `gaps.json` is committed to version control, a suppression added by one developer is respected by all CI runs and teammates.

When a suppressed gap is no longer detected (because the underlying issue was fixed), it is automatically moved to the `resolved` list on the next run.

### Prompt versioning (ADR-019)

The gap analysis prompt is versioned and stored at `benchmarks/prompts/gap-analysis-v{N}.md`. The version is referenced in `product.toml` under `[gap-analysis].prompt-version`. Findings from different prompt versions are not comparable. When the prompt version is incremented, existing suppressions are retained but flagged with warnings — the developer must re-verify each suppression against the new prompt's behaviour.
