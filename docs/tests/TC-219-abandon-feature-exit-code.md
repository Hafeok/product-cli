---
id: TC-219
title: abandon_feature_exit_code
type: exit-criteria
status: unimplemented
validates:
  features: 
  - FT-018
  adrs:
  - ADR-010
phase: 1
---

after abandoning a feature with linked tests, run `product graph check`. Assert exit code 2 (warning) not 1 (error).