---
id: TC-028
title: exit_code_broken_link
type: exit-criteria
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-009
phase: 1
---

add a feature that references a non-existent ADR. Assert exit code 1.