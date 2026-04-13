It looks like write permission to the `docs/guide/` directory keeps getting denied. Here is the complete documentation for FT-005 — Formal Specification. You can save it to `docs/guide/FT-005-formal-specification.md`:

---

## Overview

Formal Specification gives test criteria a machine-parseable layer of mathematical precision on top of their Markdown bodies. Test criteria of type `invariant` or `chaos` can embed AISP-influenced formal blocks — type definitions, invariants, scenarios, exit criteria, and evidence annotations — directly in their Markdown. Product parses these blocks, validates their syntax and value ranges, includes them in context bundles, reports them in graph health checks, and computes a formal coverage metric across the knowledge graph.

## Tutorial

### Writing your first formal block

Assume you have an existing test criterion `docs/tests/TC-020-betweenness-centrality.md` of type `invariant`. Open it and add formal blocks after the YAML front-matter:

```markdown
---
id: TC-020
title: Betweenness Centrality Always In Range
type: invariant
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-012]
phase: 3
runner: cargo-test
runner-args: "tc_020_betweenness_centrality_always_in_range"
---

## Description

Betweenness centrality must always be in [0, 1].

⟦Σ:Types⟧{
  Node≜IRI
  Centrality≜f64
}

⟦Γ:Invariants⟧{
  ∀n:Node: 0.0 ≤ betweenness(n) ≤ 1.0
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

### Verifying your formal blocks parse correctly

Run the graph health check to confirm the blocks are well-formed:

```bash
product graph check
```

If the blocks parse successfully, no errors appear for TC-020. If there is a syntax problem, you will see an `E001` error pointing to the file and describing the issue.

### Checking formal coverage

Run graph stats to see the formal coverage metric:

```bash
product graph stats
```

The output includes a line like:

```
  Formal coverage (invariant/chaos): 75%
```

This reports the percentage of `invariant` and `chaos` test criteria that contain at least one formal block.

## How-to Guide

### Add type definitions to a test criterion

1. Open the TC file.
2. Add a `⟦Σ:Types⟧` block with one type per line, using `≜` as the definition operator:

```
⟦Σ:Types⟧{
  Node≜IRI
  Role≜Leader|Follower|Learner
  ClusterState≜⟨nodes:Node+, roles:Node→Role⟩
}
```

3. Run `product graph check` to validate.

### Add invariants to a test criterion

1. Open the TC file.
2. Add a `⟦Γ:Invariants⟧` block. Each non-empty line (or group of lines separated by blank lines) becomes one invariant:

```
⟦Γ:Invariants⟧{
  ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1
}
```

3. Run `product graph check` to validate.

### Add a scenario specification

1. Open the TC file.
2. Add a `⟦Λ:Scenario⟧` block with `given≜`, `when≜`, and `then≜` fields:

```
⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader
       ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}
```

Multi-line values are supported: continuation lines are joined to the current field.

3. Run `product graph check` to validate.

### Add an evidence annotation

1. Open the TC file.
2. Add an `⟦Ε⟧` evidence block with three semicolon-separated fields:

```
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

- `δ` (delta) — confidence, a float in `[0.0, 1.0]`
- `φ` (phi) — coverage, an integer in `[0, 100]`
- `τ` (tau) — stability: `◊⁺` (stable), `◊⁻` (unstable), or `◊?` (unknown)

3. Run `product graph check` to validate.

### Fix formal block parse errors

1. Run `product graph check`.
2. Look for `E001` errors referencing formal blocks. Common causes:
   - Unclosed block delimiter (`⟦` without matching `⟧`)
   - Unclosed brace (`{` without matching `}`)
   - Unrecognised block type (e.g., `⟦X:Unknown⟧`)
   - Evidence `δ` outside `[0.0, 1.0]` or `φ` outside `[0, 100]`
3. Fix the syntax in the TC file and re-run `product graph check`.

### Suppress the W004 warning for a test criterion

The `W004` warning fires when an `invariant` or `chaos` TC has no formal blocks. To resolve it, add at least one formal block (types, invariants, scenario, exit criteria, or evidence) to the TC body.

## Reference

### Block types

| Block | Delimiter | Content format |
|---|---|---|
| Types | `⟦Σ:Types⟧{ ... }` | One `Name≜Expression` per line |
| Invariants | `⟦Γ:Invariants⟧{ ... }` | One invariant per line/paragraph |
| Scenario | `⟦Λ:Scenario⟧{ ... }` | `given≜`, `when≜`, `then≜` fields |
| Exit Criteria | `⟦Λ:ExitCriteria⟧{ ... }` | One criterion per line |
| Evidence | `⟦Ε⟧⟨δ≜N;φ≜N;τ≜S⟩` | Inline (no braces) |

### Evidence field ranges

| Field | Type | Range | Description |
|---|---|---|---|
| `δ` (delta) | `f64` | `[0.0, 1.0]` | Confidence level |
| `φ` (phi) | `u8` | `[0, 100]` | Coverage percentage |
| `τ` (tau) | symbol | `◊⁺`, `◊⁻`, `◊?` | Stability (stable, unstable, unknown) |

### Diagnostics

| Code | Tier | Condition |
|---|---|---|
| `E001` | Error | Unclosed delimiter, unclosed brace, unrecognised block type, evidence field out of range |
| `W004` | Warning | `invariant` or `chaos` TC has no formal blocks; or a formal block body is empty |
| `W006` | Warning | Evidence `δ` is below `0.7` |

### Context bundle integration

When `product context FT-XXX` assembles a bundle, it:

1. Collects all evidence blocks from linked test criteria.
2. Computes aggregate `δ` (mean of all evidence deltas).
3. Computes `φ` as the percentage of linked TCs that have at least one formal block.
4. Includes these in the bundle header as `⟦Ε⟧⟨δ≜...;φ≜...;τ≜...⟩`.

Aggregate stability uses worst-case: if any evidence block is `◊⁻`, the aggregate is `◊⁻`; otherwise if any is `◊?`, the aggregate is `◊?`; otherwise `◊⁺`.

### Graph stats

`product graph stats` reports formal coverage as a percentage:

```
Formal coverage (invariant/chaos): 75%
```

This is computed as: (number of `invariant`/`chaos` TCs with at least one formal block) / (total `invariant`/`chaos` TCs) x 100.

### Fitness function

The `formal_coverage` metric in `product metrics` uses the same ratio. It contributes to overall architectural fitness scoring.

## Explanation

### Why formal blocks?

Test criteria written in natural language are ambiguous. Two readers may interpret "centrality is always in range" differently. Formal blocks add a precise, parseable mathematical layer that:

- Provides unambiguous specification for LLM-driven implementation (the implementing agent can read `∀n:Node: 0.0 ≤ betweenness(n) ≤ 1.0` and generate a property test)
- Enables automated validation of the specification itself (range checks, structural correctness)
- Feeds aggregate confidence metrics into context bundles, so downstream agents know how well-specified a feature is

### AISP notation

The block syntax uses AISP (AI Specification Protocol) influenced notation with Unicode mathematical symbols. The delimiters `⟦` and `⟧` mark block boundaries, and `≜` serves as the definition operator. This notation was chosen to be visually distinct from Markdown and unambiguous to parse, while remaining human-readable. See ADR-016 for the design rationale behind diagnostic reporting for formal blocks.

### Evidence aggregation

Evidence blocks are not just metadata — they flow into the context bundle that `product context` and `product implement` assemble. When an implementing agent receives a bundle with `δ≜0.42`, it knows the specification is weak and should proceed cautiously. High `δ` values signal well-specified features where the agent can implement with confidence. The worst-case stability rule ensures that a single unstable specification drags the aggregate down, preventing false confidence.

### Relationship to graph health

`product graph check` validates formal blocks as part of its sweep. This means specification errors surface alongside broken links, dependency cycles, and other graph problems — there is no separate validation step. The W004 warning nudges authors toward adding formal blocks to `invariant` and `chaos` TCs, where precision matters most. The W006 warning flags low-confidence evidence, prompting authors to either improve the specification or acknowledge the uncertainty.

### File write safety (ADR-015)

All file mutations performed by Product — including updates to TC front-matter after `product verify` — use atomic writes (temp file + fsync + rename) and advisory locking on `.product.lock`. This ensures that formal block content is never corrupted by interrupted writes or concurrent Product invocations. Stale lock files from crashed processes are automatically detected and cleared. Leftover `.product-tmp.*` files are cleaned on startup.
