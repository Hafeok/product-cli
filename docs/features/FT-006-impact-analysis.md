---
id: FT-006
title: Impact Analysis
phase: 2
status: complete
depends-on:
- FT-016
adrs:
- ADR-009
- ADR-012
tests:
- TC-009
- TC-010
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
- TC-157
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
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

`product impact` performs reverse-graph reachability analysis to show the full affected set when an artifact changes.

```
product impact ADR-002    # full affected set if this decision changes
product impact FT-001     # what depends on this feature completing
product impact TC-003     # what depends on this test criterion
```

Impact analysis traverses all five edge types in reverse to find every artifact reachable from the target. The output lists affected features, ADRs, and test criteria grouped by hop distance.

This is used by:
- ADR supersession (auto-triggered when `product adr status ADR-XXX superseded` is called)
- Pre-implementation review (`product implement` step 2 references impact for drift context)
- Manual change assessment before modifying shared decisions

---

## Description

FT-006 implements reverse-graph reachability analysis via the `product impact` command (ADR-012, Capability 4). Given any artifact — an ADR, Feature, or Test Criterion — the command traverses the reverse of the knowledge graph to compute the full set of artifacts that depend on the target, grouped by hop distance. This gives developers and agents precise change-blast-radius information before modifying a shared architectural decision, superseding an ADR, or retiring a feature. Impact analysis is also triggered automatically when `product adr status ADR-XXX superseded` is called, printing the impact report before the status write is committed. The implementation operates on the same in-memory graph built from front-matter on every invocation; no separate index is needed.

## Functional Specification

### Inputs

- CLI invocation: `product impact <ARTIFACT-ID>` where `ARTIFACT-ID` is a valid Feature, ADR, or Test Criterion ID present in the graph.
- Optional flag `--format json` to request machine-readable output.
- The in-memory knowledge graph derived from all front-matter on the current invocation.

### Outputs

- **Text output (default)**: a grouped report listing direct dependents (features, ADRs, tests reachable at hop-distance 1 in the reverse graph) and transitive dependents (hop-distance ≥ 2), with each artifact's current status. The summary line highlights any passing tests in the affected set — these are the highest-urgency items when superseding a decision.
- **JSON output (`--format json`)**: a structured object with `direct` and `transitive` arrays, each containing artifact ID, type, title, and status. Suitable for CI annotation.
- **Exit code**: 0 on success regardless of whether dependents are found; 1 on error (e.g. artifact ID not found).

### State

Stateless. The reverse graph is derived on the fly from the forward graph that is built from front-matter on every invocation. No impact cache or persistent impact index is maintained.

### Behaviour

1. Product builds the in-memory knowledge graph from all front-matter in the configured directories.
2. Product constructs the reverse graph by inverting every edge: for each forward edge A → B, the reverse graph contains B → A.
3. Product performs a BFS from the target artifact node in the reverse graph, collecting all reachable nodes grouped by hop depth.
4. The output is split into direct dependents (depth 1) and transitive dependents (depth ≥ 2). Within each group, artifacts are further grouped by type (Features, ADRs, Test Criteria).
5. Passing tests that appear in the affected set are flagged in the summary line, indicating that currently passing tests may be invalidated by the change to the target artifact.
6. When invoked automatically from `product adr status ADR-XXX superseded`, the impact report is printed to stdout before the status write is performed. The write proceeds regardless of the impact report content — the report is informational, not a gate.

### Invariants

- Every artifact reachable from the target in the reverse graph appears exactly once in the output, regardless of the number of paths connecting them (deduplication by artifact ID).
- The hop-distance grouping is accurate: a node at hop distance 1 is a direct dependent, a node at hop distance ≥ 2 is a transitive dependent.
- Betweenness centrality is not recomputed for impact analysis; impact analysis uses only reverse-graph BFS (ADR-012, Capability 4).
- The impact command is read-only; it never modifies any artifact file.

### Error handling

- Target artifact ID not found in the graph → error E002 on stderr, exit code 1.
- Target artifact ID is syntactically invalid → error E005 on stderr, exit code 1.
- Graph build errors (malformed front-matter) → E001 on stderr for the affected files; the graph is built from the remaining valid files and impact analysis proceeds on the partial graph (consistent with ADR-013 error recovery behaviour).

### Boundaries

- Impact analysis traverses all five edge types in reverse (`implementedBy`, `validatedBy`, `testedBy`, `supersedes`, `depends-on`) — not a subset. A node reachable via any edge type is included in the affected set.
- The command reports the affected set but does not suggest or perform remediation. Deciding how to respond to a large impact set is left to the developer or agent.
- Impact analysis does not filter by artifact status. A `planned` feature is included in the affected set even if it has not yet been implemented.

## Out of scope

- Automatic remediation or bulk status updates for affected artifacts — impact analysis is read-only.
- Betweenness centrality ranking (`product graph central`) — a separate capability defined in ADR-012 (Capability 3) and surfaced by FT-010 / FT-016.
- Context bundle assembly for the affected set — use `product context` on individual affected features if context is needed.
- Filtering the impact set by status, phase, or domain — the full reachable set is always reported; post-processing is the caller's responsibility.
