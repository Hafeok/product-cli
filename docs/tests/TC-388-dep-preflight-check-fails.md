---
id: TC-388
title: dep_preflight_check_fails
type: scenario
status: unimplemented
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
---

DEP-005 availability check exits 1. Assert preflight report names DEP-005 as unavailable. Assert exit code 2 (warning, not error).