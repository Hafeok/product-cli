## Overview

Context Bundle is the mechanism Product uses to assemble precise, scoped context for LLM agents. Given a feature, ADR, or phase, the `product context` command traverses the knowledge graph via BFS, collects all related artifacts (ADRs, test criteria), orders them by relevance using betweenness centrality, and emits a single Markdown document. This bundle is what implementation agents, authoring sessions, and the `product implement` pipeline consume — it ensures agents receive exactly the context they need, no more and no less.

## Tutorial

### Generating your first context bundle

Pick a feature you want to understand in context. Generate its bundle at the default depth (1 hop):

```bash
product context FT-001
```

Product reads the knowledge graph from YAML front-matter, finds all ADRs and test criteria directly linked to FT-001, and prints a Markdown bundle to stdout.

### Exploring deeper relationships

Increase the traversal depth to include artifacts two hops away:

```bash
product context FT-001 --depth 2
```

At depth 2, the bundle includes not just the ADRs linked to FT-001, but also ADRs linked to *those* ADRs' sibling features, giving the agent broader architectural context.

### Saving a bundle to a file

Redirect the output to inspect or share it:

```bash
product context FT-001 --depth 2 > bundle.md
```

### Bundling an ADR

Context bundles work for ADRs too. This collects all features and test criteria linked to an ADR:

```bash
product context ADR-003
```

### Bundling an entire phase

To get context for all features in a phase at once:

```bash
product context FT-001 --phase 2 --depth 1
```

The `id` argument is ignored when `--phase` is provided — all features in that phase are bundled together.

## How-to Guide

### Get context before manual implementation

When implementing a feature without `product implement`, assemble the context bundle first:

1. Run `product context FT-XXX --depth 2` to get the full bundle.
2. Review the included ADRs — they describe decisions the implementation must respect.
3. Review the test criteria section — these define what "done" means.
4. Begin implementation informed by the bundle contents.

> **Note:** If you use `product implement FT-XXX`, do *not* also run `product context` — the pipeline assembles the bundle automatically.

### Get only ADRs for a phase (no test criteria)

When you need architectural context without test details — for example, during planning:

1. Run `product context FT-001 --phase 3 --adrs-only`.
2. The output includes only the feature descriptions and linked ADRs, omitting test criteria.

### Control ADR ordering

By default, ADRs are ordered by betweenness centrality (most connected first). To sort by ID instead:

1. Run `product context FT-001 --order id`.

This produces a deterministic, alphabetical ordering useful for diffing bundles across runs.

### Understand the three-tier ADR structure

The bundle organises ADRs in three tiers, following ADR-025:

1. **Cross-cutting ADRs** — decisions that apply to every feature (e.g., error model, file safety). Included automatically, ordered by centrality.
2. **Domain ADRs** — the top 2 ADRs by centrality for each domain the feature declares. Included automatically even if not explicitly linked.
3. **Feature-linked ADRs** — ADRs explicitly referenced in the feature's `adrs` front-matter field.

This tiering ensures agents always see foundational decisions first, then domain-relevant context, then feature-specific decisions.

## Reference

### Command syntax

```
product context <ID> [OPTIONS]
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Feature ID (e.g., `FT-001`) or ADR ID (e.g., `ADR-003`). Required, but ignored when `--phase` is provided. |

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--depth <N>` | `1` | BFS traversal depth. Controls how many hops from the root artifact to include. |
| `--phase <N>` | — | Bundle all features in the given phase. Overrides the `<ID>` argument. |
| `--adrs-only` | `false` | Exclude test criteria from the bundle. Only applies with `--phase`. |
| `--order <ORDER>` | centrality | Set to `id` to sort ADRs alphabetically instead of by betweenness centrality. |

### Output format

The bundle is a Markdown document printed to stdout with these sections:

1. **Header** — `# Context Bundle: <ID> — <title>`
2. **AISP block** — structured metadata: feature ID, phase, status, generation timestamp, linked ADR and TC IDs. For feature bundles with formal evidence blocks, an aggregate evidence line (`⟦Ε⟧`) is appended.
3. **Feature section** — the feature's body content.
4. **ADR sections** — one `## ADR-XXX — Title` section per included ADR. Superseded ADRs are annotated with `[SUPERSEDED by ADR-YYY]`.
5. **Test Criteria section** — one `### TC-XXX — Title (type)` entry per test criterion, ordered by phase then type (exit-criteria → scenario → invariant → chaos).

### Depth warning

When `--depth` is 3 or greater and the bundle exceeds 50 artifacts, a warning is printed to stderr:

```
warning: bundle contains 62 artifacts at depth 3. Consider narrowing scope.
```

### Exit behaviour

- Prints the bundle to stdout and exits 0 on success.
- Prints an error to stderr and exits 1 if the artifact ID is not found in the graph.

## Explanation

### Why bundles exist

LLM agents perform best when given precisely scoped context. Too little context and the agent misses constraints; too much and it loses focus. The context bundle solves this by leveraging the knowledge graph's structure — BFS traversal from a root artifact naturally captures the "cone of relevance" at a configurable depth.

### Graph-derived, not stored

Consistent with ADR-003 (no persistent graph store), bundles are computed fresh on every invocation. The knowledge graph is rebuilt from YAML front-matter each time. This means bundles always reflect the current state of the repository — there is no cache to invalidate or stale state to worry about.

### Centrality-based ordering

Betweenness centrality measures how often an artifact appears on shortest paths between other artifacts. ADRs with high centrality are architectural "hubs" — decisions that connect many features. Placing these first in the bundle ensures agents read the most load-bearing decisions before feature-specific ones. This ordering is the default because it produces better agent behaviour in practice; the `--order id` escape hatch exists for reproducibility.

### Three-tier ADR inclusion (ADR-025)

Not all relevant ADRs are explicitly linked in a feature's front-matter. Cross-cutting decisions (like the error model or file write safety) apply to every feature but would be tedious to link individually. Domain ADRs capture architectural context for the feature's declared domains even when the feature author didn't think to link them. The three-tier system — cross-cutting, domain (top-2 by centrality), feature-linked — ensures comprehensive coverage without requiring exhaustive manual linking.

### Relationship to `product implement`

The `product implement FT-XXX` pipeline calls the same `bundle_feature` function internally to assemble context before spawning the implementation agent. Running `product context` separately is for manual workflows, inspection, and debugging — not for use alongside `product implement`.

### Phase bundles and planning

Phase bundles (`--phase N`) concatenate individual feature bundles for all features in a phase. Combined with `--adrs-only`, this provides a compact architectural overview suitable for phase planning without the noise of individual test criteria.
