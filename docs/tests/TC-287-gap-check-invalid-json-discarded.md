---
id: TC-287
title: gap_check_invalid_json_discarded
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

inject a model response with one valid finding and one malformed finding. Assert the valid finding is in output. Assert the malformed finding is logged to stderr and discarded.