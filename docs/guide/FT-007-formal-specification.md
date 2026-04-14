## Overview

Formal Specification provides a structured notation for embedding machine-parseable type definitions, invariants, scenarios, exit criteria, and evidence blocks inside test criterion documents. The parser extracts these formal blocks from TC markdown bodies, making them available for graph validation, context bundle assembly, and fitness function metrics. This bridges the gap between human-readable specifications and automated quality checks — test criteria can express precise constraints that Product validates structurally, not just by running tests.

## Tutorial

### Writing your first formal block

Formal blocks live inside test criterion files (`docs/tests/TC-XXX-*.md`), in the body below the YAML front-matter. Let's add type definitions and an invariant to a test criterion.

1. Open an existing TC file, for example `docs/tests/TC-040-cluster-state.md`.

2. Below the front-matter, add a types block that defines the domain types under test:

   ```
   ⟦Σ:Types⟧{
     Node≜IRI
     Role≜Leader|Follower|Learner
   }
   ```

3. Add an invariant block expressing a constraint that must always hold:

   ```
   ⟦Γ:Invariants⟧{
     ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1
   }
   ```

4. Add an evidence block summarizing confidence metrics:

   ```
   ⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
   ```

5. Verify the blocks parse correctly by running graph validation:

   ```bash
   product validate
   ```

   If the blocks are malformed, you will see `E001` errors. If a block body is empty, you will see a `W004` warning.

6. Check that formal coverage is reported in graph stats:

   ```bash
   product stats
   ```

   The output includes a line like:

   ```
     Formal coverage (invariant/chaos): 85%
   ```

### Adding a scenario block

Scenario blocks use a given/when/then structure for behavioral specifications:

1. In a TC file, add:

   ```
   ⟦Λ:Scenario⟧{
     given≜cluster_init(nodes:2)
     when≜elapsed(10s)
     then≜∃n∈nodes: roles(n)=Leader
   }
   ```

2. Run `product validate` to confirm the block parses without errors.

## How-to Guide

### Add formal blocks to an invariant or chaos TC

1. Open the TC file in `docs/tests/`.
2. Confirm the front-matter `type` is `invariant` or `chaos` — formal blocks are expected on these types.
3. Add one or more formal blocks in the body (see Reference for syntax).
4. Run `product validate` — look for `E001` errors or `W004` warnings related to formal blocks.

### Check formal specification coverage

1. Run:

   ```bash
   product stats
   ```

2. Look at the `Formal coverage (invariant/chaos)` line. This is the percentage of invariant and chaos TCs that contain at least one formal block.

### Fix formal block parse errors

1. Run `product validate` and look for `E001` errors mentioning "formal block parse error".
2. Common issues:
   - **Unclosed delimiter**: a `⟦` without a matching `⟧`, or a `{` without a closing `}`.
   - **Unrecognised block type**: the block header is not one of the known types (see Reference).
   - **Evidence out of range**: `δ` must be in `[0.0, 1.0]` and `φ` must be in `[0, 100]`.
3. Fix the syntax in the TC file and re-run `product validate`.

### View formal block evidence in a context bundle

1. Run:

   ```bash
   product context FT-XXX --depth 2
   ```

2. The bundle header includes aggregate evidence metrics (`δ`, `φ`) computed from evidence blocks across all TCs linked to the feature.

## Reference

### Block types

| Block | Header | Purpose |
|-------|--------|---------|
| Types | `⟦Σ:Types⟧{ ... }` | Define domain types as `Name≜Expression` pairs |
| Invariants | `⟦Γ:Invariants⟧{ ... }` | Express constraints that must always hold |
| Scenario | `⟦Λ:Scenario⟧{ ... }` | Given/when/then behavioral specification |
| Exit Criteria | `⟦Λ:ExitCriteria⟧{ ... }` | Conditions for completion |
| Evidence | `⟦Ε⟧⟨δ≜V;φ≜V;τ≜V⟩` | Confidence, coverage, and stability metrics |

### Types block syntax

```
⟦Σ:Types⟧{
  Name≜Expression
  AnotherName≜Expression
}
```

Each line defines one type. The `≜` operator separates the type name from its expression. Trailing semicolons are stripped. Empty lines are ignored.

### Invariants block syntax

```
⟦Γ:Invariants⟧{
  ∀g:Graph, ∀n∈g.nodes: betweenness(g,n) ≥ 0.0 ∧ betweenness(g,n) ≤ 1.0
}
```

Each non-empty line (or group of contiguous non-empty lines) is one invariant. Blank lines separate distinct invariants.

### Scenario block syntax

```
⟦Λ:Scenario⟧{
  given≜precondition
  when≜trigger
  then≜expected outcome
}
```

Fields: `given≜`, `when≜`, `then≜`. All three are optional. Multi-line values are supported — continuation lines are concatenated until the next field or block end.

### Exit criteria block syntax

```
⟦Λ:ExitCriteria⟧{
  condition one
  condition two
}
```

Each non-empty line is one exit criterion.

### Evidence block syntax

```
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

| Field | Meaning | Range | Values |
|-------|---------|-------|--------|
| `δ` | Confidence | `[0.0, 1.0]` | Decimal float |
| `φ` | Coverage | `[0, 100]` | Integer |
| `τ` | Stability | — | `◊⁺` (stable), `◊⁻` (unstable), `◊?` (unknown) |

Evidence blocks do not use braces — they use angle brackets `⟨...⟩` with semicolon-separated fields.

### Diagnostic codes

| Code | Severity | Meaning |
|------|----------|---------|
| `E001` | Error | Formal block parse error — unclosed delimiter, unrecognised block type, or evidence field out of range |
| `W004` | Warning | Empty block body (`⟦Type⟧{}`) or missing formal blocks on invariant/chaos TCs |
| `W006` | Warning | Low-confidence specification — evidence `δ` below 0.7 threshold |

### Where formal blocks are used

- **Parsing**: Extracted from TC bodies during artifact loading (`src/parser.rs`). Stored on the `TestCriterion.formal_blocks` field.
- **Validation**: `product validate` re-parses formal blocks and reports `E001`/`W004` diagnostics.
- **Context bundles**: `product context` aggregates evidence blocks across linked TCs to compute bundle-level `δ` and `φ` values.
- **Stats**: `product stats` reports formal coverage as the percentage of invariant/chaos TCs that have at least one formal block.
- **Fitness functions**: The `φ` (phi) architectural fitness metric uses formal block presence on invariant/chaos TCs.

## Explanation

### Why a custom notation?

Test criteria need to express both prose descriptions (readable by humans and LLMs) and precise, structured constraints (parseable by tooling). YAML front-matter handles metadata (id, type, status, validates), but constraints like "for all nodes in a graph, betweenness centrality is in [0, 1]" don't fit naturally into key-value pairs.

The AISP-influenced notation provides a middle ground: it lives inline in the markdown body, is visually distinct from prose thanks to Unicode delimiters (`⟦`, `⟧`, `≜`), and can be extracted by regex-based parsing without a full markdown AST. This follows the decision in ADR-004 to use markdown as the sole document format — formal blocks are markdown-compatible content, not a separate file format.

### Evidence as a quality signal

Evidence blocks (`⟦Ε⟧`) attach quantitative confidence metrics directly to specifications. The three fields capture orthogonal quality dimensions:

- **δ (delta)** — how confident we are in the specification's correctness. Values below 0.7 trigger a `W006` warning during validation, flagging specifications that may need review.
- **φ (phi)** — what percentage of the relevant domain the specification covers. Used at the graph level to compute formal coverage stats.
- **τ (tau)** — whether the specification is stable (`◊⁺`), unstable (`◊⁻`), or unknown (`◊?`). When aggregating across multiple evidence blocks, the system uses worst-case stability: any unstable block makes the aggregate unstable.

Context bundles aggregate these metrics across all TCs linked to a feature, giving the LLM (and the developer) a single confidence signal for the feature's specification quality.

### Formal coverage and graph health

The `product validate` command checks that invariant and chaos TCs contain formal blocks — these test types are expected to have precise, machine-parseable constraints. A missing formal block on an invariant TC produces a `W004` warning, nudging authors to express their invariants in the structured notation rather than relying solely on prose.

The `product stats` command reports the aggregate formal coverage percentage. This metric feeds into the architectural fitness functions (see `src/metrics.rs`), making formal specification coverage a measurable aspect of repository health alongside test coverage and gap density.
