---
id: TC-215
title: exit_code_broken_link
type: exit-criteria
status: unimplemented
validates:
  features: 
  - FT-010
  - FT-014
  - FT-026
  adrs:
  - ADR-009
phase: 1
---

add a feature that references a non-existent ADR. Assert exit code 1.