---
id: TC-386
title: dep_impact_transitive
type: scenario
status: unimplemented
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
---

FT-003 depends-on FT-001; FT-001 uses DEP-001. Assert `product impact DEP-001` includes FT-003 in transitive dependents.