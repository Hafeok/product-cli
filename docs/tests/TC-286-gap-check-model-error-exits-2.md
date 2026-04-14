---
id: TC-286
title: gap_check_model_error_exits_2
type: exit-criteria
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

inject a network failure for the model call. Assert exit code 2 (warning), not 1 (new gaps). Assert error appears on stderr.