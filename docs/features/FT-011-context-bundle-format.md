---
id: FT-011
title: Context Bundle Format
phase: 1
status: complete
depends-on: []
adrs:
- ADR-006
- ADR-008
- ADR-012
tests:
- TC-016
- TC-017
- TC-018
- TC-019
- TC-020
- TC-024
- TC-025
- TC-026
- TC-041
- TC-042
- TC-043
- TC-044
- TC-045
- TC-046
- TC-047
- TC-048
- TC-049
- TC-050
- TC-051
- TC-052
- TC-053
- TC-054
- TC-158
- TC-201
- TC-202
- TC-203
- TC-205
- TC-232
- TC-233
- TC-234
- TC-235
- TC-236
- TC-237
- TC-238
- TC-249
domains:
- api
- data-model
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

The context command assembles a deterministic markdown bundle. Order is always: feature → ADRs (by ID ascending) → test criteria (by phase, then type: exit-criteria, scenario, invariant, chaos).

The bundle opens with an AISP-influenced formal header block (see ADR-011) that an agent can parse without reading the full document. It declares the bundle's identity, all linked artifact IDs, and aggregate evidence metrics derived from the test criteria evidence blocks.

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

---

```

---

## Description

`product context FT-XXX` assembles a deterministic markdown bundle containing a feature, its linked ADRs, and its linked test criteria. The bundle is the primary input to LLM implementation agents (ADR-006). It opens with an AISP-influenced formal header block (`⟦Ω:Bundle⟧`) that declares the bundle's identity and aggregate evidence metrics, allowing an agent to assess specification quality before reading full content. YAML front-matter is stripped from all included sections; formal blocks in test criteria are preserved verbatim.

## Functional Specification

### Inputs

- Feature ID (e.g. `FT-001`) — required positional argument to `product context`
- `--depth N` — optional BFS traversal depth (default 1); depth 2 includes transitive ADRs and dependencies (ADR-012)
- `--order id` — optional flag to override centrality-based ADR ordering and sort by ID ascending

### Outputs

A single markdown document on stdout with this fixed structure:
1. Bundle header: `# Context Bundle: FT-XXX — Title`
2. AISP formal block `⟦Ω:Bundle⟧{...}` listing feature ID, phase, status, generated timestamp, all ADR IDs (`implementedBy`), all TC IDs (`validatedBy`)
3. Aggregate evidence line `⟦Ε⟧⟨δ≜N;φ≜N;τ≜◊⁺⟩` (mean confidence across linked TCs, formal-block coverage percentage)
4. `---` separator
5. Feature content (front-matter stripped)
6. ADRs in three-tier order: cross-cutting ADRs (by betweenness centrality) → domain ADRs (top-2 per declared domain by centrality) → feature-linked ADRs; superseded ADRs annotated `[SUPERSEDED by ADR-XXX]`
7. Dependency artifacts if any linked
8. Test criteria ordered by phase then type: exit-criteria → invariant → chaos → absence → scenario → benchmark

If `--depth N ≥ 3` and the resulting bundle exceeds 50 nodes, a warning is emitted to stderr; the bundle is still produced.

### State

Stateless. No data is retained between invocations. The graph is rebuilt from YAML front-matter on each call (ADR-003). The `⟦Ε⟧` aggregate metrics are computed fresh each time from the linked TCs' evidence blocks.

### Behaviour

1. Parse all artifact files to build the in-memory knowledge graph.
2. Locate the requested feature node; return an error if not found.
3. BFS from the feature node to the configured depth, collecting all reachable ADR and TC IDs.
4. Apply three-tier ADR ordering: all cross-cutting ADRs first (by centrality), then domain ADRs (top-2 per feature domain by centrality), then directly linked feature ADRs.
5. Sort TCs by phase ascending then by type sort key (exit-criteria first, benchmark last).
6. Compute `δ` (mean of all evidence block `δ` values) and `φ` (percentage of TCs with any formal block).
7. Render the bundle: strip front-matter from each section, preserve formal blocks verbatim.
8. Deduplication: any node reachable via multiple paths appears once, at its first-encountered position.

### Invariants

- Bundle output is deterministic: two invocations of `product context FT-XXX` on the same repository produce identical output (barring clock-only differences in the `generated` timestamp field).
- YAML front-matter is never present in bundle output (TC-017).
- The `⟦Ω:Bundle⟧` header lists every ADR ID and TC ID that appears in the bundle body (TC-018).
- Superseded ADRs appear with `[SUPERSEDED by ADR-XXX]` annotation, never silently dropped (TC-019).
- Cross-cutting ADRs are always included regardless of explicit feature links (ADR-025).

### Error handling

- Unknown feature ID: exits with a non-zero status and a message indicating the feature was not found.
- Malformed front-matter in any artifact: reported as E001 via the graph build phase; the bundle command inherits the graph's parse-error collection and may report diagnostics to stderr before producing (possibly partial) output.
- Depth ≥ 3 with > 50 nodes: warning to stderr, bundle produced (not blocked).

### Boundaries

- Does not invoke any LLM or network service; purely a local file-reading and rendering operation.
- Does not write any files; output is stdout only.
- Does not infer `depends-on` edges or additional links beyond what front-matter declares.
- Token budget management is the caller's responsibility — Product assembles a complete bundle; truncation is not performed (ADR-006).

## Out of scope

- Streaming or incremental context delivery (the bundle is assembled in full before output).
- Per-model token budget trimming or context window management.
- Rendering formats other than markdown (JSON output of the bundle content is not provided by this feature).
- Authoring or mutating any artifact; this is a read-only assembly command.
