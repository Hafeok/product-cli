---
id: TC-025
title: sparql_untested_features
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-011
  - FT-016
  - FT-014
  adrs:
  - ADR-008
phase: 1
runner: cargo-test
runner-args: "tc_025_sparql_untested_features"
---

load a graph where FT-002 has no `pm:validatedBy` triples. Execute a query for features with no test criteria. Assert FT-002 appears in the result and FT-001 (which has tests) does not.