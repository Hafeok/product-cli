---
id: TC-282
title: gap_check_no_gaps
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

run `product gap check ADR-001` against a fixture with full TC coverage. Assert exit code 0 and an empty findings array.