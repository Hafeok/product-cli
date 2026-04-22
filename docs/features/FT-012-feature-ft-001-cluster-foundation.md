---
id: FT-012
title: 'Feature: FT-001 — Cluster Foundation'
phase: 1
status: complete
depends-on: []
adrs:
- ADR-001
tests:
- TC-001
- TC-002
- TC-003
- TC-004
- TC-163
domains: []
domains-acknowledged:
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

[full content of FT-001-cluster-foundation.md, front-matter stripped]

---