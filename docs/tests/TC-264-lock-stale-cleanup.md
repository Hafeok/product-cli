---
id: TC-264
title: lock_stale_cleanup
type: scenario
status: unimplemented
validates:
  features: 
  - FT-004
  - FT-005
  adrs:
  - ADR-015
phase: 1
---

create a `.product.lock` file with a non-existent PID. Run any write command. Assert the command succeeds (stale lock was detected and cleared).