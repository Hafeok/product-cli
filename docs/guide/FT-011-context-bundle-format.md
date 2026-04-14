## Overview

The context bundle is Product's primary output format for delivering targeted knowledge-graph context to LLM agents and engineers. Instead of dumping an entire repository into a prompt, `product context` assembles a deterministic Markdown document containing exactly one feature, its linked ADRs, and its linked test criteria — typically 3,000–8,000 tokens. The bundle includes a machine-parseable header block so agents can extract identity and evidence metadata without reading the full document.

## Tutorial

### Assembling your first context bundle

Start with a repository that has at least one feature linked to ADRs and test criteria.

1. List available features to find one to work with:

   ```bash
   product feature list
   ```

2. Generate a context bundle for a feature:

   ```bash
   product context FT-001
   ```

3. Examine the output. The bundle begins with a formal header block:

   ```markdown
   # Context Bundle: FT-001 — Cluster Foundation

   ⟦Ω:Bundle⟧{
     feature≜FT-001:Feature
     phase≜1:Phase
     status≜InProgress:FeatureStatus
     generated≜2026-04-11T09:00:00Z
     implementedBy≜⟨ADR-001,ADR-002,ADR-003,ADR-006⟩:Decision+
     validatedBy≜⟨TC-001,TC-002,TC-003,TC-004⟩:TestCriterion+
   }
   ⟦Ε⟧⟨δ≜0.92;φ≜75;τ≜◊⁺⟩
   ```

   After the header, the bundle contains the full content of the feature, each linked ADR, and each linked test criterion — in that order.

4. Notice the ordering: YAML front-matter is stripped from all sections, and ADRs are ordered by betweenness centrality (most structurally important first) by default.

### Using depth for transitive context

When a feature shares foundational ADRs with adjacent features, you can widen the context window:

1. Generate a depth-2 bundle to include transitive context:

   ```bash
   product context FT-001 --depth 2
   ```

2. Compare with the default depth-1 bundle. At depth 2, the bundle also includes:
   - Other features that share ADRs with FT-001
   - Features that FT-001 depends on, along with their ADRs and tests
   - ADRs and tests of those transitive features

3. Each artifact appears only once in the bundle, even if reachable via multiple paths.

## How-to Guide

### Assemble context for agent-driven implementation

1. Run the context command for the target feature:

   ```bash
   product context FT-003 --depth 2
   ```

2. Pipe the output into your agent's context window, or save it to a file:

   ```bash
   product context FT-003 --depth 2 > bundle.md
   ```

3. The agent can parse the `⟦Ω:Bundle⟧` header to extract artifact IDs without reading the full document.

**Note:** If you are using `product implement FT-003`, the pipeline assembles the context bundle automatically. Do not also run `product context` — that duplicates the context.

### Get ADR-ordered bundles by ID instead of centrality

By default, ADRs in a depth-1 bundle are ordered by betweenness centrality (most important first). To get ID-ascending order instead:

```bash
product context FT-001 --order id
```

### Check the impact of changing a decision before modifying a bundle's contents

1. Run impact analysis on the ADR you plan to change:

   ```bash
   product impact ADR-002
   ```

2. Review the output for directly and transitively affected features and tests.

3. Pay attention to passing tests that may be invalidated — these are the highest-urgency items.

### Inspect which ADRs are most structurally important

1. View the top ADRs by betweenness centrality:

   ```bash
   product graph central
   ```

2. Narrow the list:

   ```bash
   product graph central --top 5
   ```

3. Or see the full ranking:

   ```bash
   product graph central --all
   ```

## Reference

### `product context` command

```
product context <FEATURE-ID> [OPTIONS]
```

Assembles a deterministic Markdown context bundle for the given feature.

| Flag / Option | Default | Description |
|---|---|---|
| `<FEATURE-ID>` | *(required)* | The feature to assemble context for (e.g., `FT-001`) |
| `--depth <N>` | `1` | BFS traversal depth. `1` = direct links only. `2` = transitive context. |
| `--order <MODE>` | `centrality` | ADR ordering in the bundle. `centrality` = betweenness centrality descending. `id` = ID ascending. |

**Output structure** (in order):

1. **Header block** — AISP formal block with bundle identity, linked artifact IDs, and evidence metrics
2. **Feature content** — full feature description (front-matter stripped)
3. **ADR sections** — linked ADRs, ordered by centrality (default) or ID
4. **Test criteria sections** — ordered by phase, then by type: exit-criteria, scenario, invariant, chaos

**Depth semantics:**

| Depth | Includes |
|---|---|
| 1 | Seed feature, its direct ADRs, its direct test criteria |
| 2 | Depth-1 artifacts + features sharing those ADRs, depends-on features and their ADRs/tests |
| 3+ | Recursive expansion. Warning emitted if bundle exceeds 50 nodes. |

**Deduplication:** Artifacts reachable via multiple paths appear once, at first-encountered position.

**Large bundle warning:** At depth 3+, if the bundle exceeds 50 artifacts, a warning is emitted to stderr:

```
Bundle contains N artifacts at depth 3. Consider narrowing scope.
```

The bundle is still produced — the warning does not block output.

### Bundle header format

```markdown
⟦Ω:Bundle⟧{
  feature≜FT-001:Feature
  phase≜1:Phase
  status≜InProgress:FeatureStatus
  generated≜2026-04-11T09:00:00Z
  implementedBy≜⟨ADR-001,ADR-002⟩:Decision+
  validatedBy≜⟨TC-001,TC-002⟩:TestCriterion+
}
⟦Ε⟧⟨δ≜0.92;φ≜75;τ≜◊⁺⟩
```

| Field | Type | Description |
|---|---|---|
| `feature` | Feature ID | The seed feature of this bundle |
| `phase` | Integer | The feature's implementation phase |
| `status` | FeatureStatus | Current status (e.g., `InProgress`, `Complete`) |
| `generated` | ISO 8601 timestamp | When the bundle was assembled |
| `implementedBy` | List of ADR IDs | All ADRs included in the bundle |
| `validatedBy` | List of TC IDs | All test criteria included in the bundle |
| `⟦Ε⟧` | Evidence block | Aggregate evidence metrics from test criteria |

### Superseded ADR handling

Superseded ADRs are replaced by their successors in the bundle. A superseded ADR does not appear in the output. The supersession chain is queryable via `product adr show` but does not propagate into context bundles.

### `product graph central` command

```
product graph central [OPTIONS]
```

| Flag / Option | Default | Description |
|---|---|---|
| `--top <N>` | `10` | Number of top ADRs to display |
| `--all` | *(off)* | Show full ranked list |

**Output format:**

```
Rank  ID       Centrality  Title
1     ADR-001  0.847       Rust as Implementation Language
2     ADR-002  0.731       openraft for Cluster Consensus
3     ADR-006  0.612       Oxigraph for RDF Projection
```

### `product impact` command

```
product impact <ARTIFACT-ID>
```

Accepts any artifact ID (ADR, feature, or test criterion). Performs reverse-graph BFS to compute the full set of affected artifacts.

**Output sections:**

- **Direct dependents** — features and tests one hop away in the reverse graph
- **Transitive dependents** — features and tests reachable via dependency chains
- **Summary** — count of affected artifacts, with passing tests flagged as potentially invalidated

### Related configuration

Bundle assembly reads graph structure from YAML front-matter in paths configured in `product.toml`:

```toml
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
```

## Explanation

### Why targeted bundles instead of full-repository context

A project with 40 features, 30 ADRs, and 80 test criteria produces over 200,000 tokens if dumped in full. This exceeds the practical working window of most models and degrades output quality even when it fits. The context bundle solves this by assembling only the artifacts relevant to a single feature — typically 3,000–8,000 tokens. Nothing relevant is omitted (every linked ADR and test is included), and nothing irrelevant is included (ADR-006).

### Deterministic assembly

Two invocations of `product context FT-001` with the same graph state produce identical output. This makes bundles cacheable, auditable, and reproducible. The ordering rules (centrality for ADRs, phase-then-type for tests) are fixed and deterministic given the same graph topology.

### Centrality-based ADR ordering

Not all ADRs in a bundle are equally important. ADR-001 (Rust as implementation language) is linked to nearly every feature and acts as a structural bridge in the graph. ADR-007 (checklist generation) may apply to only one feature. Betweenness centrality — computed via Brandes' algorithm (ADR-012) — quantifies this structural importance without requiring human curation. The most foundational decisions appear first in the bundle so an agent reading top-to-bottom encounters them before peripheral decisions.

### BFS depth as a scope control

Depth-1 (the default) preserves backward compatibility and keeps bundles small. Depth-2 is useful when an agent is implementing a feature that shares foundational decisions with adjacent features — the transitive context surfaces ADRs and tests that would otherwise require separate queries. Depth-3 and beyond risk pulling in most of the graph, which is why the 50-node warning exists (ADR-012).

### The header block as a machine-readable manifest

The `⟦Ω:Bundle⟧` header uses AISP-influenced formal notation (ADR-012) so that an agent can extract the bundle's identity, all linked artifact IDs, and evidence metrics in a single parse pass — without scanning the full document. This is particularly valuable for agents that need to decide whether a bundle is relevant before committing to reading it.

### Relationship to the implementation pipeline

When `product implement FT-XXX` is invoked, the pipeline assembles the context bundle internally and passes it to the spawned agent. Running `product context` separately in this workflow would duplicate the context. Use `product context` directly when you need the bundle for manual review, for piping into a different agent, or for auditing what context an implementation task would receive.
