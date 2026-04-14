---
id: TC-305
title: verify_unimplemented_no_runner_blocks
type: scenario
status: unimplemented
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

All TCs have no `runner` field. Assert feature goes to in-progress (unimplemented blocks completion). Distinct from `status: unrunnable` which is an explicit acknowledgement that does not block.