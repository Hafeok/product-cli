---
id: TC-245
title: impact_transitive
type: scenario
status: unimplemented
validates:
  features: 
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
---

FT-007 depends-on FT-001; FT-001 linked to ADR-002. Assert `product impact ADR-002` includes FT-007 in transitive dependents.