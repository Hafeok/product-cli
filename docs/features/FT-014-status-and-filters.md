---
id: FT-014
title: Status and Filters
phase: 2
status: complete
depends-on:
- FT-016
- FT-017
adrs:
- ADR-003
- ADR-007
- ADR-008
- ADR-009
- ADR-012
tests:
- TC-009
- TC-010
- TC-021
- TC-022
- TC-023
- TC-024
- TC-025
- TC-026
- TC-027
- TC-028
- TC-029
- TC-030
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
- TC-159
- TC-181
- TC-209
- TC-210
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
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

`product status` provides a summary view of project health by phase, coverage, and dependency state.

```
product status                   # summary: features by phase and status
product status --phase 1         # phase 1 detail with test coverage
product status --untested        # features with no linked test criteria
product status --failing         # features with one or more failing tests
```

### Test Filters

```
product test untested            # features with no linked tests
product test list --failing      # tests currently failing
```

### Git Awareness

When regenerating the checklist, warn if modified files are uncommitted. This prevents stale checklist state from being committed alongside unrelated changes.

---

## Description

`product status` delivers a summary view of project health grouped by phase, including per-phase gate state (OPEN/LOCKED based on exit-criteria TC pass rates), coverage, and dependency state. Companion filter commands (`--untested`, `--failing`) allow targeted queries. The feature depends on FT-016 (graph model) for data and FT-017 (checklist) for the generated artifact that status drives.

## Functional Specification

### Inputs

- `product status` — no arguments: shows all phases with feature counts and gate state
- `product status --phase N` — shows full detail for phase N including individual exit-criteria TCs and their pass/fail/unimplemented state
- `product status --untested` — lists features with no linked test criteria
- `product status --failing` — lists features with one or more failing TCs
- `product test untested` — equivalent view for untested features
- `product test list --failing` — equivalent view for failing TCs
- Optional git-awareness: at checklist regeneration time, warns if modified files are uncommitted

### Outputs

- Structured terminal output grouped by phase, e.g.:
  ```
  Phase 1 — Cluster Foundation  [OPEN — exit criteria: 2/4 passing]
    FT-001  Cluster Foundation     complete
    FT-002  mTLS Node Comms        complete
  Phase 2 — Products and IAM  [LOCKED — Phase 1 exit criteria: TC-007, TC-012 not passing]
  ```
- Exit code 0 on success; no special exit codes for filtered results (filtering is informational, not a CI gate)
- `--format json` for structured output where supported

### State

Stateless. The graph is rebuilt from front-matter on each invocation (ADR-003). `product status` reads feature and TC status fields from YAML front-matter; it does not maintain its own state.

### Behaviour

1. Build the in-memory graph from all artifact files.
2. Compute topological sort of features by their `depends-on` DAG (ADR-012, Kahn's algorithm).
3. Group features by phase number.
4. For each phase, compute the phase gate: a phase is OPEN if all exit-criteria TCs for features in that phase have `status: passing`; LOCKED otherwise. If no exit-criteria TCs exist for a phase the gate is satisfied (TC-234).
5. `--untested`: return features where `graph.tests_for_feature(f)` is empty.
6. `--failing`: return features where at least one linked TC has `status: failing`.
7. Git-awareness: when `product checklist generate` is triggered from status, warn if the working tree has uncommitted modifications.

### Invariants

- Phase gate state is always derived from TC `status` fields in front-matter; it is never stored separately (ADR-003).
- A phase with no exit-criteria TCs reports as OPEN (not LOCKED) — backward-compatible with migrations where TCs haven't been written yet (TC-234).
- `product status` output includes every feature in the graph; no feature is silently omitted.
- LOCKED phases name the specific failing or unimplemented exit-criteria TCs (TC-237).

### Error handling

- Malformed front-matter or broken graph links are reported via `product graph check` diagnostics at build time; `product status` inherits parse errors from the graph.
- If the topological sort detects a cycle in `depends-on` edges, it falls back to alphabetical ordering and the cycle is reported as E003 by `product graph check`.

### Boundaries

- Does not modify any artifact files; read-only.
- Git-awareness is advisory (a warning), not a hard gate — `product status` never blocks on uncommitted files.
- Does not run test criteria; it reads TC `status` from front-matter as set by `product verify`.

## Out of scope

- Running or executing tests (that is `product verify`).
- Modifying feature or TC status (those are write commands on the respective artifact types).
- Providing build system or CI integration beyond exit codes (that is ADR-009 / FT-010).
