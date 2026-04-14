---
id: TC-212
title: sparql_untested_features
type: scenario
status: unimplemented
validates:
  features: 
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-008
phase: 1
---

load a graph where FT-002 has no `pm:validatedBy` triples. Execute a query for features with no test criteria. Assert FT-002 appears in the result and FT-001 (which has tests) does not.