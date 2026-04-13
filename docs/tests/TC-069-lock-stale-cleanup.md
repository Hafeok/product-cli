---
id: TC-069
title: lock_stale_cleanup
type: scenario
status: passing
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
runner: cargo-test
runner-args: "tc_069_lock_stale_cleanup"
---

create a `.product.lock` file with a non-existent PID. Run any write command. Assert the command succeeds (stale lock was detected and cleared).