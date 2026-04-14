## Overview

Core Concepts defines the three artifact types that make up Product's knowledge graph — Features (`FT-XXX`), Architectural Decision Records (`ADR-XXX`), and Test Criteria (`TC-XXX`) — along with their relationships and the derived graph that connects them. Every other capability in Product builds on these primitives: context bundles assemble them, gap analysis checks their completeness, and the implement pipeline consumes them. Understanding core concepts is prerequisite to using any part of the tool.

## Tutorial

### Step 1: Create your first feature

```bash
product feature new --title "User Authentication"
```

Product assigns the next available ID automatically. If no features exist yet, this creates `FT-001`. The file is written to `docs/features/FT-001-user-authentication.md` with YAML front-matter pre-populated.

### Step 2: Create an ADR that supports the feature

```bash
product adr new --title "JWT for Session Tokens"
```

This creates `ADR-001`. Open the file and add `FT-001` to its `applies-to` list in the front-matter so the graph links the decision to the feature.

### Step 3: Create a test criterion

```bash
product test new --title "Login returns valid JWT" --type scenario
```

This creates `TC-001`. Edit its front-matter to link it to the feature and ADR:

```yaml
validates:
  features:
    - FT-001
  adrs:
    - ADR-001
```

### Step 4: See the graph

```bash
product graph show FT-001
```

You should see `FT-001` connected to `ADR-001` via `implementedBy` and to `TC-001` via `validatedBy`. The graph is built fresh from front-matter on every invocation — there is no separate database to keep in sync.

### Step 5: Assemble a context bundle

```bash
product context FT-001 --depth 2
```

This outputs a single markdown document containing the feature, its linked ADRs, and its linked test criteria — ready for injection into an LLM context window. Front-matter `---` blocks are stripped from the output.

## How-to Guide

### Create a feature, ADR, or test criterion

```bash
product feature new --title "My Feature"
product adr new --title "My Decision"
product test new --title "My Test" --type scenario
```

IDs are assigned sequentially. The next ID is always `max(existing) + 1` — gaps in the sequence are never filled.

### Link artifacts together

Edit the YAML front-matter of the source artifact. Relationships are declared in the source:

- **Feature → ADR**: add the ADR ID to the feature's `implemented-by` list
- **Feature → TC**: add the TC ID to the feature's `validated-by` list
- **ADR → TC**: add the TC ID to the ADR's `tested-by` list
- **ADR → ADR**: use `supersedes` to chain decisions

### Inspect relationships for an artifact

```bash
product graph show FT-001
```

The graph is bidirectional. Querying any artifact shows both its outgoing and incoming edges.

### Export the graph for external tools

```bash
product graph rebuild
```

This writes `index.ttl` (Turtle/RDF format) as a snapshot. Product itself never reads this file — it is for external tooling such as SPARQL query engines.

### Retire an artifact without breaking references

Set `status: abandoned` in the artifact's front-matter. Do not delete the file or renumber other artifacts. IDs are permanent — external references in commit messages, code comments, and chat remain valid.

## Reference

### Artifact ID format

| Prefix | Artifact type | Example |
|--------|--------------|---------|
| `FT`   | Feature | `FT-001` |
| `ADR`  | Architectural Decision Record | `ADR-001` |
| `TC`   | Test Criterion | `TC-001` |

IDs are zero-padded to three digits. Prefixes are configurable in `product.toml`.

### ID assignment rules

- IDs are assigned sequentially by `product feature/adr/test new`.
- Next ID = `max(existing IDs) + 1`. Gaps are never filled.
- Creating an artifact with an ID that already exists returns an error — the existing file is never overwritten.
- Once assigned, an ID is permanent. Artifacts are never renumbered.

### Relationship edges

| Source | Edge | Target |
|--------|------|--------|
| Feature | `implementedBy` | ADR |
| Feature | `validatedBy` | TestCriterion |
| ADR | `testedBy` | TestCriterion |
| ADR | `supersedes` | ADR |

Edges are declared in the source artifact's front-matter. The in-memory graph makes all edges traversable in both directions.

### Test criterion types

| Type | Purpose |
|------|---------|
| `scenario` | A concrete sequence of actions and expected outcomes |
| `invariant` | A property that must always hold |
| `chaos` | Behaviour under failure or degraded conditions |
| `exit-criteria` | A gate that must pass before a feature is considered complete |

### Context bundle output

`product context FT-XXX --depth N` assembles a markdown document containing:

1. The feature itself (front-matter stripped)
2. All linked ADRs (up to `--depth` hops)
3. All linked test criteria

Output is deterministic — same inputs produce same output. YAML front-matter `---` blocks are removed so the bundle is clean markdown suitable for LLM injection.

### Configuration (`product.toml`)

Relevant keys for core concepts:

- Artifact file paths (where `docs/features/`, `docs/adrs/`, `docs/tests/` are located)
- ID prefixes (default: `FT`, `ADR`, `TC` — configurable per project)
- Phase and status thresholds

## Explanation

### Why a derived graph with no persistent store?

Product rebuilds its in-memory graph from YAML front-matter on every command invocation (ADR-003). This means the graph is always consistent with the files — there is no cache to invalidate, no migration to run, and no risk of stale state. The cost is a full scan of all artifact files on each run, but for the expected scale (hundreds of artifacts, not millions) this completes in milliseconds.

### Why numeric IDs instead of slugs or UUIDs?

ADR-005 documents this decision. Numeric IDs are short enough to type in a commit message (`FT-001`), unambiguous, and stable — they never change even if the artifact title changes. UUIDs are globally unique but unreadable in context. Slugs are readable but unstable if the title is edited. The zero-padding ensures correct alphabetical sort in file listings.

### Why markdown with YAML front-matter?

ADR-004 chose CommonMark markdown because it renders natively on GitHub and GitLab, requires no conversion for LLM context injection, and supports the prose-heavy content of ADRs and features. YAML front-matter provides the structured metadata (IDs, links, status) that the graph needs, while the markdown body carries the human-readable content.

### Why Rust?

ADR-001 chose Rust to meet the single-binary deployment constraint — no runtime, no installer, no dependency on Node.js or Python. The choice also aligns Product with PiCloud's toolchain, enabling shared CI patterns and potential library sharing (particularly Oxigraph for SPARQL).

### The context bundle as primary output

Everything in Product exists to make context bundles accurate and complete. Features organize capability. ADRs capture decisions. Test criteria define verification. The graph connects them. The context bundle assembles the relevant subset into a single document that an LLM can consume. If a bundle is missing information, that is a signal that the graph is incomplete — which is exactly what `product gap check` detects.
