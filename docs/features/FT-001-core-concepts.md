---
id: FT-001
title: Core Concepts
phase: 1
status: complete
depends-on: []
adrs:
- ADR-001
- ADR-004
- ADR-005
tests:
- TC-001
- TC-002
- TC-003
- TC-004
- TC-011
- TC-012
- TC-013
- TC-014
- TC-015
- TC-156
domains:
- data-model
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
---

### Artifact Types

**Feature (`FT-XXX`)** — A unit of product capability. Corresponds to a section of a PRD. Declares its phase, status, linked ADRs, and linked test criteria. A feature is the primary navigation unit of the knowledge graph: everything else is reachable from it.

**Architectural Decision Record (`ADR-XXX`)** — A single architectural decision. Declares context, decision, rationale, rejected alternatives, and the features it applies to. An ADR may apply to multiple features. An ADR may supersede or be superseded by another ADR.

**Test Criterion (`TC-XXX`)** — A single verifiable assertion about system behaviour. A test criterion has a type (scenario, invariant, chaos, exit-criteria), is linked to one or more features and one or more ADRs, and belongs to a phase. Test criteria are extracted from ADRs during migration — they are not co-located with the decisions they verify.

### Relationships

```
Feature ──── implementedBy ────► ADR
Feature ──── validatedBy ───────► TestCriterion
ADR     ──── testedBy ──────────► TestCriterion
ADR     ──── supersedes ────────► ADR
```

Edges are declared in the *source* artifact's front-matter. The derived graph is bidirectional — every edge is traversable in both directions by the CLI.

### The Derived Graph

Product reads all front-matter declarations on every command invocation and builds an in-memory graph. There is no persistent graph store. The graph is always consistent with the files. `product graph rebuild` writes `index.ttl` as a snapshot for external tooling, but this file is never read by Product itself.

### The Context Bundle

A context bundle is a single markdown document containing a feature, all its linked ADRs, and all its linked test criteria — assembled in a deterministic order and formatted for direct injection into an LLM context window. This is the primary output of Product. Everything else in the tool exists to make context bundles accurate and complete.

---

---

## Description

Product's knowledge graph is built from three first-class artifact types — Features (`FT-XXX`), Architectural Decision Records (`ADR-XXX`), and Test Criteria (`TC-XXX`) — expressed as CommonMark markdown files with YAML front-matter. Each artifact file is self-describing: its identity, relationships, and graph position are all declared in its own front-matter (ADR-002). The graph is derived on every invocation by reading all front-matter; there is no persistent store (ADR-003). The primary output consumed by LLM agents is a context bundle: a single deterministically ordered markdown document containing a feature, all its linked ADRs, and all its linked test criteria (ADR-006). This feature defines the three artifact types, their relationship model, the derived-graph invariant, and the context-bundle concept.

## Functional Specification

### Inputs

- A directory tree of `.md` files conforming to the repository layout (see FT-002). Each file's YAML front-matter declares artifact type (inferred from `id` prefix), ID, status, and outgoing edges.
- CLI invocation specifying a command and optional artifact ID as target.

### Outputs

- An in-memory graph built from all front-matter, available to all commands for the lifetime of the invocation.
- For context-assembly commands: a markdown context bundle containing the target feature, its linked ADRs ordered by betweenness centrality, and its linked test criteria (ADR-012).
- For list and show commands: text or JSON representation of one or more graph nodes.

### State

Stateless between invocations. The in-memory graph is rebuilt from files on every command invocation (ADR-003). The only persisted artefact is `index.ttl`, written by `product graph rebuild` for external tooling consumption; it is never read by Product itself.

### Behaviour

1. On every invocation, Product scans the configured directories (`docs/features/`, `docs/adrs/`, `docs/tests/` by default) and parses YAML front-matter from every `.md` file.
2. Each file is classified by its `id` prefix: `FT-` → Feature, `ADR-` → Architectural Decision Record, `TC-` → Test Criterion.
3. Edges are extracted from `adrs`, `tests`, `features`, `supersedes`, `superseded-by`, and `depends-on` front-matter fields and added to the in-memory graph. Every declared edge is traversable in both directions.
4. The assembled graph is handed to the requested command. Navigation commands (`feature list`, `feature show`) query the graph directly. Context commands assemble bundles via BFS traversal (ADR-012). Impact commands traverse the reverse graph (ADR-012).
5. When `product graph rebuild` is called, the in-memory graph is serialised to `index.ttl` as an RDF/Turtle snapshot.
6. Front-matter is stripped before any file content is injected into a context bundle (ADR-004). Code blocks, tables, and headings are preserved verbatim (TC-011, TC-012).

### Invariants

- Every `id` field must be unique across all artifact files of the same prefix type (E005 on duplicate detection).
- Every edge declared in front-matter references an artifact that exists in the repository; unresolvable references are reported as E002 (broken link).
- The `depends-on` edges between Feature nodes must form a directed acyclic graph; cycles are reported as E003.
- Betweenness centrality values for all ADR nodes lie in [0.0, 1.0] (ADR-012, TC-007 in FT-007's formal block).
- The context bundle output contains no YAML front-matter delimiters (`---`) or raw front-matter fields — only the stripped markdown body (TC-011).

### Error handling

- Malformed YAML front-matter → E001 (parse error), reported on stderr with file path and line number; the file is skipped and remaining files continue to parse (ADR-013).
- Missing required `id` field → E006, structured error with field name and file path.
- Broken link to a non-existent artifact → E002, reported by `product graph check` with exit code 1 (ADR-009).
- Dependency cycle in `depends-on` DAG → E003, exit code 1.
- All errors go to stderr; stdout is reserved for command output so that `product context FT-001 > bundle.md` produces a clean file even when warnings are present (ADR-013).

### Boundaries

- The three artifact types (Feature, ADR, Test Criterion) are the only first-class graph nodes. Free-form markdown outside these three types is not parsed or tracked.
- The formal block notation inside TC files (ADR-011) is a separate parsing concern; this feature covers only the structural graph model and front-matter schema for the three types.
- Context bundle assembly depth is 1 by default (direct edges only); transitive BFS at greater depth is enabled by `--depth N` (ADR-012) and is part of FT-006/FT-010.

## Out of scope

- Persistent graph storage: the graph is always derived from files; no SQLite, RDF store, or daemon is in scope (ADR-003).
- Authoring commands (`product feature new`, `product adr new`, `product test new`) — covered by FT-004.
- Schema versioning and migration of front-matter fields — covered by FT-008.
- The formal block notation (`⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, etc.) inside TC files — covered by FT-005 and FT-007.
- Impact analysis traversal and betweenness centrality commands — covered by FT-006.
- CI exit-code conventions for graph health commands — covered by FT-010.
