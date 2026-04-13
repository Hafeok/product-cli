---
id: TC-027
title: exit_code_clean
type: exit-criteria
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-009
phase: 1
---

run `product graph check` on a fully consistent repository. Assert exit code 0.