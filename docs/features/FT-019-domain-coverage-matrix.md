---
id: FT-019
title: Domain Coverage Matrix
phase: 5
status: complete
depends-on: []
adrs:
- ADR-025
- ADR-026
tests:
- TC-132
- TC-133
- TC-134
- TC-135
- TC-136
- TC-137
- TC-138
- TC-139
- TC-140
- TC-141
- TC-142
- TC-143
- TC-144
- TC-145
- TC-146
- TC-147
- TC-148
- TC-149
- TC-150
- TC-151
domains:
- api
- data-model
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
---

`product graph coverage` produces the feature × domain coverage matrix — the portfolio-level view of architectural completeness at scale.

```
product graph coverage

                    sec  stor  cons  net  obs  err  iam  sched  api  data
FT-001 Cluster       ✓    ✓     ✓    ✓    ✓    ✓    ✓    ✓     ✓    ✓
FT-002 Products      ✓    ✓     ·    ✓    ✓    ✓    ✓    ·     ✓    ·
FT-003 RDF Store     ~    ✓     ·    ·    ✓    ✓    ~    ·     ✓    ✓
FT-009 Rate Limit    ✗    ✗     ·    ✓    ✗    ✗    ✗    ·     ✓    ·

Legend:
  ✓  covered      — feature has a linked ADR in this domain
  ~  acknowledged — domain acknowledged with explicit reasoning, no linked ADR
  ·  not declared — feature does not declare this domain (may still apply)
  ✗  gap          — feature declares domain but has no coverage
```

`product preflight FT-XXX` produces the single-feature view of the same data, with specific ADRs named and resolution commands printed:

```
product preflight FT-009

━━━ Cross-Cutting ADRs (must acknowledge all) ━━━━━━━━━━━━━━

  ✓  ADR-001  Rust as implementation language          [linked]
  ✓  ADR-013  Error model and diagnostics              [linked]
  ✗  ADR-038  Observability requirements               [not acknowledged]

━━━ Domain Coverage ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  networking  ✓  ADR-004 (linked), ADR-006 (linked)
  security    ✗  no coverage — top-2 by centrality: ADR-011, ADR-019

━━━ To resolve ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  product feature link FT-009 --adr ADR-038
  product feature acknowledge FT-009 --domain security --reason "..."
```

Domain coverage is integrated into the `product implement` pipeline as Step 0 — pre-flight must be clean before context assembly or agent invocation. See ADR-026.

---

---

## Description

FT-019 provides two views of architectural domain coverage: the portfolio-level matrix (`product graph coverage`) and the single-feature pre-flight report (`product preflight FT-XXX`). Domain coverage is governed by ADR-025 (domain vocabulary and scope classification) and ADR-026 (pre-flight as mandatory Step 0 in `product implement`). Coverage gaps that are not linked or acknowledged block the implement pipeline.

## Functional Specification

### Inputs

- `product graph coverage`: no required arguments; optional `--domain D` to filter to one column, `--format json` for machine-readable output
- `product preflight FT-XXX`: feature ID; reads graph plus `product.toml` domain vocabulary and cross-cutting ADR set
- Feature front-matter fields: `domains` (declared concern areas), `adrs` (linked ADRs), `domains-acknowledged` (explicit reasoning for gap closure)
- ADR front-matter fields: `domains`, `scope` (`cross-cutting` | `domain` | `feature-specific` | `platform`)
- `product.toml`: `[domains]` vocabulary table

### Outputs

- `product graph coverage`: feature × domain matrix rendered as a table with symbols:
  - `✓` covered (feature has a linked ADR in that domain)
  - `~` acknowledged (domain acknowledged with reasoning, no linked ADR)
  - `·` not declared (feature does not declare this domain)
  - `✗` gap (feature declares domain but has no coverage)
- `product preflight FT-XXX`: structured report showing cross-cutting ADR status (linked / not acknowledged), domain coverage per declared domain, and resolution commands to run
- Exit code: `product preflight` exits non-zero (code 1) if any gaps remain; `product graph check` reports gaps as W010/W011 warnings (exit 2)

### State

Stateless. Domain coverage is computed fresh from front-matter on every invocation (ADR-003). No coverage state is persisted.

### Behaviour

1. Build the in-memory graph.
2. `product graph coverage`: for each feature, for each domain in the `product.toml` vocabulary, classify the feature's coverage as `✓`, `~`, `·`, or `✗` by checking `feature.domains`, `feature.adrs` (cross-referenced against ADR `domains` fields), and `feature.domains-acknowledged`.
3. `product preflight FT-XXX`:
   a. Check all cross-cutting ADRs (scope `cross-cutting`): each must be linked or acknowledged by the feature.
   b. For each declared domain in `feature.domains`: check top-2 domain-scoped ADRs by centrality; each must be linked or acknowledged.
   c. Emit a structured report with resolution commands (`product feature link`, `product feature acknowledge`).
   d. Return exit code 1 if any gaps remain, 0 if clean.
4. `product implement FT-XXX`: preflight runs as Step 0 before context assembly; implementation is blocked if preflight exits non-zero (ADR-026). Unlike gap analysis, preflight coverage gaps cannot be bypassed — they must be resolved or acknowledged.
5. Domain vocabulary validation: a feature declaring a domain not in `product.toml` produces E012 (hard error, TC-139).
6. `domains-acknowledged` entries with empty or whitespace-only reasoning produce E011 (hard error) — reasoning is mandatory (ADR-025).

### Invariants

- Cross-cutting ADRs appear in every feature's context bundle regardless of explicit links (TC-132).
- `domains-acknowledged` requires non-empty reasoning — an empty acknowledgement is E011, not a soft warning.
- A feature declaring an unknown domain (not in `product.toml` vocabulary) is E012, a hard error (TC-139).
- `product preflight` results are not cached — each invocation re-reads the graph (ADR-003, ADR-026).
- Domain ADR inclusion in context bundles is limited to top-2 by centrality per domain to avoid context explosion (ADR-025).

### Error handling

- E011: `domains-acknowledged` entry with empty reasoning; blocks commands that require a clean preflight.
- E012: domain declared in feature front-matter not present in `product.toml` vocabulary (TC-139).
- W010: cross-cutting ADR not linked or acknowledged by a feature; reported by `product graph check` (exit 2).
- W011: feature declares a domain with existing domain-scoped ADRs but no coverage.
- `product implement` block: preflight failures always block implementation — the message names each unresolved gap with the exact resolution command.

### Boundaries

- Does not infer domain applicability from prose content; domain membership is declared explicitly in front-matter.
- Does not auto-link or auto-acknowledge gaps; resolution always requires an explicit developer action.
- Pre-flight analysis is graph traversal only — no LLM calls (ADR-026).

## Out of scope

- Generating domain suggestions from feature prose (possible future enhancement per ADR-026, not implemented).
- Enforcing domain coverage on legacy features that predate ADR-025 (retroactive enforcement is waived via `domains-acknowledged` entries).
- Tracking which engineer acknowledged a domain gap (no attribution metadata).
