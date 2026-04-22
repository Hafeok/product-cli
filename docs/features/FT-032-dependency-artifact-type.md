---
id: FT-032
title: Dependency Artifact Type
phase: 3
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-030
tests:
- TC-381
- TC-382
- TC-383
- TC-384
- TC-385
- TC-386
- TC-387
- TC-388
- TC-389
- TC-390
- TC-391
- TC-392
- TC-393
- TC-394
- TC-395
- TC-396
- TC-397
- TC-398
- TC-399
- TC-400
- TC-401
- TC-403
- TC-678
domains:
- api
- data-model
domains-acknowledged:
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
---

## Description

First-class `DEP-XXX` artifact type for external dependencies (ADR-030). Six types: library, service, api, tool, hardware, runtime. Integrates with preflight (availability checks), context bundles (interface contracts), impact analysis (`product impact DEP-XXX`), gap analysis (G008), and produces a bill of materials (`product dep bom`).
