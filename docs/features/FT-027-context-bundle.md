---
id: FT-027
title: Context Bundle
phase: 5
status: complete
depends-on: []
adrs:
- ADR-026
tests:
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
- TC-678
- TC-679
domains:
- api
domains-acknowledged:
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
---

{BUNDLE}
```

The test status table is generated fresh at invocation time — the agent sees which TCs are currently passing and which are not.

---

```

---

## Description

The Context Bundle feature covers two related capabilities: (1) `product context FT-XXX [--depth N]` assembles a deterministic markdown bundle of a feature, its linked ADRs, its linked test criteria, and (at depth 2) transitive graph neighbours — the primary input to LLM agents (ADR-006); (2) `product preflight FT-XXX` performs domain coverage analysis before authoring or implementation begins, checking that all cross-cutting ADRs are linked or acknowledged and that each declared domain has at least one governing ADR (ADR-026). The test status table embedded in the bundle is generated fresh at invocation time so the agent always sees the current TC pass/fail state. `product graph coverage` provides the portfolio-level coverage matrix.

## Functional Specification

### Inputs

- **`product context FT-XXX [--depth N] [--measure] [--order id|centrality]`**: feature ID; depth controls BFS traversal depth (default 1); `--measure` records bundle metrics to `metrics.jsonl`; `--order` controls ADR ordering within the bundle
- **`product preflight FT-XXX`**: feature ID; checks cross-cutting ADR coverage and declared domain coverage
- **`product feature acknowledge FT-XXX --adr ADR-XXX --reason "..."` or `--domain DOMAIN --reason "..."`**: closes a preflight gap by recording an acknowledgement in feature front-matter
- **`product graph coverage [--domain DOMAIN] [--format json]`**: no required arguments; `--domain` filters to a single domain column
- **`product.toml` `[domains]`**: the domain taxonomy; cross-cutting ADRs identified by `scope: cross-cutting` in their front-matter
- **`product.toml` `[author]`**: prompt version configuration used when preflight is called inside an authoring session

### Outputs

- **Context bundle**: deterministic markdown to stdout containing: bundle header with artifact manifest (`⟦Ω:Bundle⟧`), feature body, ADRs ordered by betweenness centrality descending, test criteria with current status, and a test status summary table
- **`product preflight` report**: structured coverage report listing cross-cutting ADR gaps, domain gaps, and resolution commands; exit 0 (clean) or exit 1 (gaps present)
- **`product graph coverage` matrix**: ASCII grid of feature × domain coverage symbols (✓ linked, ~ acknowledged, · not applicable, ✗ gap); `--format json` produces machine-readable output
- **`metrics.jsonl` entry** (when `--measure` is used): bundle measurement record with depth-1-adrs, depth-2-adrs, tcs, domains, tokens-approx

### State

The context bundle and preflight report are computed on demand from the current graph state (ADR-003). Acknowledgements are persisted in feature front-matter (atomic writes per ADR-015) and become part of the knowledge graph. The `metrics.jsonl` append is the only persistent side-effect of `product context --measure`.

### Behaviour

1. **Context assembly**: `product context FT-XXX` reads the feature file, collects all linked ADRs (ordered by betweenness centrality descending), collects all linked TCs with their current status, and renders them as a markdown bundle. At depth 2, BFS follows all edge types to collect transitive graph neighbours; each node is deduplicated. Superseded ADRs are excluded from the bundle — only current accepted decisions appear.
2. **Test status table**: TC statuses are read from TC front-matter at invocation time. The table always reflects the most recently written status, not a cached value.
3. **Preflight analysis**: `product preflight FT-XXX` traverses the full cross-cutting ADR set and checks (a) each is either in `feature.adrs` or has an entry in `feature.domains-acknowledged`, and (b) each domain in `feature.domains` has at least one governing ADR linked or acknowledged. Domain ADR coverage uses the top-2 ADRs by centrality per domain (following ADR-025 scoping).
4. **Acknowledgement**: `product feature acknowledge` writes to `domains-acknowledged` in feature front-matter atomically and re-runs preflight validation to confirm the gap is closed.
5. **`product implement` integration**: preflight runs as Step 0 in the `product implement` pipeline. Preflight failures always block `product implement` — domain coverage gaps cannot be suppressed, only acknowledged with explicit reasoning.
6. **Depth ≥ 3 warning**: if the resulting bundle exceeds 50 nodes at depth ≥ 3, a warning is emitted to stderr. The bundle is still produced.

### Invariants

- Two invocations of `product context FT-XXX` with the same graph state produce identical output — deterministic assembly is required for auditability (ADR-006).
- Superseded ADRs never appear in context bundles. Their successors appear instead.
- Preflight failures always block `product implement`. Unlike gap findings, preflight domain gaps cannot be suppressed — only acknowledged with explicit per-domain reasoning.
- `--measure` records bundle size metrics to `metrics.jsonl` without modifying any artifact front-matter.

### Error handling

- **Feature not found**: exits 1 with E001 listing the feature ID.
- **Depth ≥ 3, bundle > 50 nodes**: warning on stderr, bundle produced without blocking.
- **Preflight with unacknowledged gaps**: exits 1; the report names each unresolved gap and provides the acknowledgement command to run. `product implement` is blocked until preflight exits 0.
- **`product feature acknowledge` without `--reason`**: exits 1 with an error message — a reason is required for the acknowledgement to be recorded (reasoning is the point, not suppression).

### Boundaries

- Context bundles contain only document graph artifacts (features, ADRs, TCs). Source code files are never included in a context bundle — that is the domain of drift detection.
- Token budget management is the agent's responsibility. Product assembles a complete, accurate bundle; it does not truncate to fit a context window (ADR-006).
- Preflight does not start an agent session. It is a knowledge check that a harness or developer runs before starting a session.

## Out of scope

- Context truncation to fit a model's context window (agent's responsibility per ADR-006)
- Including source code files in context bundles (drift detection is the separate feature for that)
- Caching context bundles between invocations (rebuilt fresh every time per ADR-003)
- Streaming context bundle assembly (the full bundle is assembled then written to stdout)
- LLM-suggested domain assignments for preflight (deterministic graph traversal only in v1)
