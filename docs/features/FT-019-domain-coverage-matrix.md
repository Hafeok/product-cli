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
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
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