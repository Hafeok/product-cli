---
id: TC-238
title: status_phase_detail
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

run `product status --phase 1`. Assert output lists all exit-criteria TCs for phase 1 with their individual pass/fail status.