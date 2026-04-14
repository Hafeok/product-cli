---
id: TC-361
title: link_tests_adr_scope
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

run `product migrate link-tests --adr ADR-002`. Assert only TCs linked to ADR-002 are updated. TCs for ADR-006 unchanged.