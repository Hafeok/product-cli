---
id: TC-313
title: verify_wrapper_script
type: scenario
status: unimplemented
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

TC configured with `runner: bash`, `runner-args: ["scripts/test-harness/raft.sh"]`. Script exits 0. Assert TC status `passing`. Script exits 1. Assert TC status `failing`. Product has no knowledge of what the script does internally.