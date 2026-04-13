It looks like the write permission was denied twice. The file at `docs/guide/FT-011-context-bundle-format.md` already exists but is empty (1 line). Could you grant write permission so I can save the documentation, or would you prefer I output the content directly here for you to copy?

Here's the complete documentation for FT-011 — Context Bundle Format:

---

## Overview

The context bundle is Product's primary output format for delivering targeted knowledge-graph context to LLM agents and engineers. Rather than dumping an entire repository into a prompt, `product context` assembles a deterministic Markdown document containing exactly the feature, ADRs, and test criteria relevant to a task — typically 3,000–8,000 tokens. The bundle opens with a machine-parseable formal header block (AISP-influenced) so an agent can extract identity, status, and evidence metrics without reading the full document. This design is governed by ADR-006 (Context Bundle as the Primary LLM Interface).

## Tutorial

### Generating your first context bundle

Run the context command with a feature ID:

```bash
product context FT-001
```

Product rebuilds the knowledge graph from front-matter, walks one hop from FT-001, and prints a Markdown bundle to stdout. The output contains:

1. A formal header block with the feature's ID, phase, status, and linked artifacts
2. The feature body (YAML front-matter stripped)
3. All linked ADRs, ordered by betweenness centrality (most structurally important first)
4. All linked test criteria, ordered by phase then type

### Saving a bundle to a file

Redirect stdout to capture the bundle:

```bash
product context FT-001 > bundle.md
```

### Including transitive context

If the feature shares ADRs with adjacent features or has `depends-on` relationships, increase the traversal depth:

```bash
product context FT-001 --depth 2
```

At depth 2, the bundle includes artifacts reachable through shared ADRs and dependency chains — giving an agent the full surrounding context for a complex implementation task.

### Reading the formal header

The top of every feature bundle contains a block like this:

```
⟦Ω:Bundle⟧{
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜2026-04-13T12:34:56.789Z
  implementedBy≜⟨ADR-001,ADR-002⟩:Decision+
  validatedBy≜⟨TC-001,TC-002⟩:TestCriterion+
}
```

An agent can parse this block to discover all linked artifact IDs before reading the body. If test criteria contain formal evidence blocks, an aggregate evidence line follows:

```
⟦Ε⟧⟨δ≜0.92;φ≜85;τ≜◊⁺⟩
```

Where `δ` is average confidence, `φ` is coverage percentage, and `τ` is stability.

## How-to Guide

### Bundle a single feature

```bash
product context FT-003
```

Returns the feature, its ADRs (ordered by centrality), and its test criteria.

### Bundle a single ADR

```bash
product context ADR-002
```

Returns the ADR with all linked features and tests. No formal header block is included for ADR bundles.

### Bundle all features in a phase

```bash
product context --phase 1
```

Assembles bundles for every feature with `phase: 1`, concatenated into a single output.

### Bundle a phase with ADRs only (no tests)

```bash
product context --phase 1 --adrs-only
```

Useful when you need the decision context but not the test criteria.

### Get transitive context for a complex feature

```bash
product context FT-003 --depth 2
```

Depth 2 follows edges two hops from the seed: through shared ADRs to sibling features, through `depends-on` edges to prerequisite features and their artifacts.

### Order ADRs by ID instead of centrality

```bash
product context FT-001 --order id
```

By default, ADRs are ordered by betweenness centrality (most structurally important first). Use `--order id` for ascending ID order.

### Combine options

```bash
product context --phase 2 --depth 2 --order id --adrs-only
```

## Reference

### Command syntax

```
product context <ID> [OPTIONS]
product context --phase <N> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Feature ID (e.g., `FT-001`) or ADR ID (e.g., `ADR-002`) to bundle |

### Options

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--depth <N>` | `usize` | `1` | BFS traversal depth from the seed node |
| `--phase <N>` | `u32` | — | Bundle all features in phase N (overrides positional ID) |
| `--adrs-only` | flag | `false` | Exclude test criteria from output (only with `--phase`) |
| `--order <ORDER>` | `string` | — | ADR ordering: `id` for ID-ascending; omit for centrality-descending |

### Output format

The bundle is printed to stdout as GitHub-flavored Markdown:

```
# Context Bundle: FT-001 — Feature Title

⟦Ω:Bundle⟧{
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜<RFC 3339 timestamp>
  implementedBy≜⟨ADR-001,ADR-002⟩:Decision+
  validatedBy≜⟨TC-001,TC-002⟩:TestCriterion+
}
⟦Ε⟧⟨δ≜0.92;φ≜85;τ≜◊⁺⟩

---

## Feature: FT-001 — Feature Title

[Feature body, front-matter stripped]

---

## ADR-001 — ADR Title

**Status:** Accepted

[ADR body, front-matter stripped]

---

## Test Criteria

### TC-001 — Test Title (scenario)

[Test body, front-matter stripped]
```

### Formal header fields

| Field | Type | Description |
|-------|------|-------------|
| `feature` | Feature ID | The seed feature |
| `phase` | Integer | Phase number from feature front-matter |
| `status` | FeatureStatus | Current feature status |
| `generated` | RFC 3339 | UTC timestamp of bundle generation |
| `implementedBy` | Decision+ | Comma-separated ADR IDs included in the bundle |
| `validatedBy` | TestCriterion+ | Comma-separated TC IDs included in the bundle |

### Evidence block symbols

| Symbol | Field | Meaning |
|--------|-------|---------|
| `δ` | delta | Average confidence across test evidence [0.00–1.00] |
| `φ` | phi | Coverage percentage [0–100] |
| `◊⁺` | tau | Stable |
| `◊≃` | tau | Rebalancing |
| `◊⁻` | tau | Drifting |
| `◊?` | tau | Unknown |

The evidence block is only emitted when at least one test criterion has a formal evidence block.

### Section ordering

1. Feature (always first)
2. ADRs — three tiers: cross-cutting (by centrality), domain-scoped (top 2 per domain by centrality), then feature-linked (by centrality or ID)
3. Test criteria — sorted by phase ascending, then by type: exit-criteria, scenario, invariant, chaos

### Superseded ADRs

Superseded ADRs are replaced by their successors in the bundle. A superseded ADR does not appear unless it has no successor linked. When it does appear, it carries a `[SUPERSEDED by ADR-XXX]` annotation and its status line reads `**Status:** Superseded by ADR-XXX`.

### Depth behaviour

| Depth | What is included |
|-------|-----------------|
| 1 (default) | Seed feature + direct ADRs + direct test criteria |
| 2 | Depth 1 + features sharing those ADRs + `depends-on` features + their ADRs and tests |
| 3+ | Continues BFS expansion; warning emitted to stderr if bundle exceeds 50 artifacts |

BFS traverses both forward and reverse edges. Nodes reachable via multiple paths appear once (first-encountered position). The seed node is always first.

### Depth warning

At depth 3 or greater, if the bundle contains more than 50 artifacts, a warning is emitted to stderr:

```
warning: bundle contains N artifacts at depth D. Consider narrowing scope.
```

The bundle is still produced — the warning does not block output.

## Explanation

### Why bundles instead of full-repo context

A project with 40 features, 30 ADRs, and 80 test criteria produces over 200,000 tokens — past the practical working window of most models and past the point where signal-to-noise is useful. A single feature bundle typically produces 3,000–8,000 tokens of precisely targeted context. Empirically, agents produce better output from 5K tokens of targeted context than from 200K tokens of mixed context (ADR-006).

### Why deterministic assembly

Two invocations of `product context FT-001` produce identical output (modulo the `generated` timestamp). This makes bundles cacheable, auditable, and reproducible. Determinism comes from fixed ordering rules — centrality-descending for ADRs, phase-then-type for tests — applied to the same graph structure.

### Why centrality-based ADR ordering

Not all ADRs in a bundle are equally important. ADR-001 (Rust as implementation language) may be linked to every feature, making it a structural bridge in the knowledge graph. ADR-007 (checklist generation) may apply to a single feature. Betweenness centrality (Brandes' algorithm, ADR-012) quantifies this structural importance without human curation. An agent reading the bundle top-to-bottom encounters the most foundational decisions first. The `--order id` flag is available when stable ordering is preferred over importance ordering.

### Why the formal header block

The `⟦Ω:Bundle⟧` header is designed so an agent can parse bundle metadata without reading the full document. It declares the feature identity, all linked artifact IDs, and aggregate evidence metrics in a compact, structured format inspired by AISP notation (ADR-011). This enables workflows where an agent inspects the header to decide whether to read the full bundle or request a different scope.

### Why BFS depth is opt-in

Depth-1 bundles are compact and cover the common case: implementing a single feature with its direct decisions and tests. Depth-2 bundles are significantly larger because they pull in sibling features and transitive dependencies. Making depth-2 the default was rejected (ADR-012) because the transitive-context use case is less common and the larger bundle may exceed what an agent needs. Depth is opt-in so the caller controls the trade-off between completeness and size.

### Relationship to `product implement`

The `product implement FT-XXX` pipeline calls `product context` internally to assemble the bundle before passing it to the spawned agent. When using `implement`, do not also run `product context` — that would duplicate the context. Use `product context` directly when you need the bundle for manual workflows: pasting into a prompt, attaching to a system message, or piping into another tool.

### Graph traversal details

The BFS implementation (ADR-012) traverses both forward edges (e.g., `implementedBy`, `depends-on`) and reverse edges (e.g., features that share an ADR). This bidirectional traversal is what makes depth-2 useful: starting from FT-001, it can reach FT-004 through a shared ADR-002 even if there is no direct `depends-on` edge between them. Deduplication ensures each artifact appears exactly once regardless of how many paths lead to it.
