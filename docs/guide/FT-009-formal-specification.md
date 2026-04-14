## Overview

Formal Specification provides a structured notation for embedding machine-readable type definitions, invariants, scenarios, exit criteria, and evidence blocks inside test criterion documents. Product parses these formal blocks from YAML front-matter bodies, validates them during graph checks, aggregates evidence metrics into context bundles, and reports diagnostics when blocks are malformed or missing. This gives teams a way to express precise, verifiable constraints alongside their natural-language specifications.

## Tutorial

This tutorial walks you through adding formal blocks to a test criterion and verifying that Product parses them correctly.

### Step 1: Create a test criterion with formal blocks

Create a new test criterion file in `docs/tests/` (or use `product test new`). In the body below the YAML front-matter, add formal blocks using the AISP-influenced notation:

```markdown
---
id: TC-200
title: Leader election invariant
type: invariant
status: unknown
validates:
  features:
    - FT-001
  adrs: []
phase: 1
runner: cargo-test
runner-args: "tc_200_leader_election_invariant"
---

⟦Σ:Types⟧{
  Node≜IRI
  Role≜Leader|Follower|Learner
}

⟦Γ:Invariants⟧{
  ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1
}

⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:3)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader
}

⟦Ε⟧⟨δ≜0.90;φ≜85;τ≜◊⁺⟩
```

### Step 2: Run graph check to validate

```bash
product graph check
```

If your blocks are well-formed, the check passes cleanly. If there are problems, you will see E001 errors or W004 warnings pointing to the specific file and issue.

### Step 3: View formal coverage in stats

```bash
product graph stats
```

Look for the `Formal coverage (invariant/chaos)` line. This shows the percentage of invariant and chaos test criteria that contain at least one formal block.

### Step 4: Generate a context bundle

```bash
product context FT-001 --depth 2
```

The context bundle header includes an aggregated evidence block (`⟦Ε⟧⟨...⟩`) computed from all linked test criteria that contain evidence blocks. This gives downstream consumers (LLMs, reviewers) a confidence summary for the feature.

## How-to Guide

### Add an evidence block to a test criterion

1. Open the test criterion file in `docs/tests/`.
2. Append an evidence block at the end of the body:
   ```
   ⟦Ε⟧⟨δ≜0.85;φ≜80;τ≜◊?⟩
   ```
3. Run `product graph check` to validate the values are in range.

### Add type definitions

1. Add a `⟦Σ:Types⟧` block to the test criterion body:
   ```
   ⟦Σ:Types⟧{
     NodeId≜UUID
     Status≜Active|Inactive|Draining
   }
   ```
2. Each line inside the braces defines one type as `Name≜Expression`.

### Add invariants

1. Add a `⟦Γ:Invariants⟧` block:
   ```
   ⟦Γ:Invariants⟧{
     ∀n:Node: reachable(n) → heartbeat(n) < 30s
   }
   ```
2. Each non-empty line (or group of lines separated by blank lines) becomes one invariant.

### Add a scenario

1. Add a `⟦Λ:Scenario⟧` block with `given≜`, `when≜`, and `then≜` fields:
   ```
   ⟦Λ:Scenario⟧{
     given≜cluster_init(nodes:2)
     when≜network_partition(duration:5s)
     then≜∃n∈nodes: roles(n)=Leader
   }
   ```
2. All three fields are optional but at least one should be present.

### Fix E001 errors

1. Run `product graph check` and note the E001 diagnostic.
2. Common causes:
   - **Out-of-range evidence values**: `δ` must be in `[0.0, 1.0]`, `φ` must be in `[0, 100]`.
   - **Unclosed block delimiters**: ensure every `⟦` has a matching `⟧` and every `{` has a matching `}`.
   - **Unrecognised block type**: only `Σ:Types`, `Γ:Invariants`, `Λ:Scenario`, `Λ:ExitCriteria`, and `Ε` are valid.
3. Fix the block in the test criterion file and re-run `product graph check`.

### Fix W004 warnings

W004 appears when:
- An invariant or chaos test criterion has no formal blocks at all.
- A formal block has an empty body (`⟦Γ:Invariants⟧{}`).

Add the appropriate formal blocks or remove empty block shells.

### Fix W006 warnings

W006 fires when an evidence block has `δ < 0.7`. Improve the specification confidence or acknowledge the low score by raising the delta value once the specification is strengthened.

## Reference

### Block types

| Block | Syntax | Content |
|-------|--------|---------|
| Types | `⟦Σ:Types⟧{ ... }` | Type definitions, one per line: `Name≜Expression` |
| Invariants | `⟦Γ:Invariants⟧{ ... }` | Invariant expressions, separated by blank lines |
| Scenario | `⟦Λ:Scenario⟧{ ... }` | Fields: `given≜`, `when≜`, `then≜` (all optional) |
| Exit Criteria | `⟦Λ:ExitCriteria⟧{ ... }` | One criterion per line |
| Evidence | `⟦Ε⟧⟨...⟩` | Inline fields: `δ≜`, `φ≜`, `τ≜` separated by `;` |

### Evidence fields

| Field | Name | Range | Description |
|-------|------|-------|-------------|
| `δ` | Specification confidence | `0.0`–`1.0` | How confident the specification is correct |
| `φ` | Coverage completeness | `0`–`100` | Percentage of the domain covered |
| `τ` | Stability signal | `◊⁺` / `◊⁻` / `◊?` | Stable, unstable, or unknown |

### Diagnostic codes

| Code | Severity | Condition |
|------|----------|-----------|
| E001 | Error | Out-of-range evidence value (`δ` outside `[0.0, 1.0]`, `φ` > 100) |
| E001 | Error | Unclosed block delimiter (`⟦` without `⟧`, `{` without `}`) |
| E001 | Error | Unrecognised block type |
| W004 | Warning | Invariant/chaos test has no formal blocks |
| W004 | Warning | Empty block body |
| W006 | Warning | Evidence `δ` below 0.7 threshold |

### CLI commands that use formal blocks

| Command | How formal blocks are used |
|---------|---------------------------|
| `product graph check` | Parses all test criteria for formal blocks; reports E001/W004/W006 diagnostics |
| `product graph stats` | Reports formal coverage percentage (invariant/chaos tests with blocks) |
| `product context FT-XXX` | Aggregates evidence from linked tests into the bundle header |

### Evidence aggregation in context bundles

When `product context` generates a bundle, it:

1. Collects all evidence blocks from test criteria linked to the feature.
2. Computes `δ` as the arithmetic mean of all evidence `δ` values.
3. Computes `φ` as `(count of tests with formal blocks / total linked tests) × 100`.
4. Emits an `⟦Ε⟧⟨δ≜...;φ≜...;τ≜◊⁺⟩` line in the bundle header (only if at least one evidence block exists).

### Graph check exit codes

| Code | Meaning |
|------|---------|
| 0 | Clean — no errors, no warnings |
| 1 | Errors found (includes formal block E001 errors) |
| 2 | Warnings only (includes W004, W006) |

### Output format

`product graph check` supports `--format json` for machine-readable output:

```bash
product graph check --format json
```

## Explanation

### Why formal blocks?

Natural-language test criteria are useful for humans but ambiguous for automated tooling. Formal blocks provide a structured overlay that can be parsed, validated, and aggregated without losing the readability of the surrounding prose. The notation is influenced by AISP (as described in ADR-016) and uses Unicode mathematical symbols to remain visually distinct from Markdown.

### Where formal blocks live

Formal blocks are embedded in the body of test criterion files (`docs/tests/TC-XXX-*.md`), below the YAML front-matter. They are parsed at graph-load time by the `formal.rs` module and stored alongside the test criterion in the in-memory graph. The graph is rebuilt from source files on every invocation (ADR-003), so formal blocks are always up to date.

### Block type design

The five block types map to distinct specification concerns:

- **Types** (`Σ`) declare the vocabulary — what entities and value sets exist.
- **Invariants** (`Γ`) state what must always be true — system-wide constraints.
- **Scenarios** (`Λ:Scenario`) describe a specific before/after sequence — given/when/then.
- **Exit Criteria** (`Λ:ExitCriteria`) list conditions that must hold for a feature to be considered complete.
- **Evidence** (`Ε`) captures metadata about the specification itself — how confident, how complete, how stable.

### Evidence as a health signal

Evidence blocks serve two purposes. First, during `graph check`, they flag under-specified tests (W006 for low delta). Second, during `context` bundle generation, they give the consuming agent a quantitative signal about the reliability of the specification it is working from. A bundle with `δ≜0.95;φ≜100` carries more weight than one with `δ≜0.50;φ≜30`.

### Relationship to the ID scheme (ADR-005)

Formal blocks reference artifacts by their prefixed numeric IDs (`FT-XXX`, `ADR-XXX`, `TC-XXX`). The stable, permanent ID scheme defined in ADR-005 ensures that formal block references remain valid even as artifact titles change or artifacts are retired.

### Diagnostic philosophy

The parser separates errors (E001) from warnings (W004, W006). Errors indicate structurally broken blocks that cannot be interpreted. Warnings indicate missing or weak specifications that should be improved but do not block the pipeline. This mirrors the graph check exit code convention: exit 1 for errors, exit 2 for warnings-only.
