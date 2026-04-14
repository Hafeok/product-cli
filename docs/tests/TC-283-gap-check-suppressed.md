---
id: TC-283
title: gap_check_suppressed
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

add a suppression for a known gap to `gaps.json`. Run analysis. Assert exit code 0. Assert the finding appears in output with `"suppressed": true`.