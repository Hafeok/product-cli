---
id: TC-313
title: verify_wrapper_script
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_313_verify_wrapper_script
last-run: 2026-04-18T10:41:51.294040135+00:00
last-run-duration: 0.2s
---

TC configured with `runner: bash`, `runner-args: ["scripts/test-harness/raft.sh"]`. Script exits 0. Assert TC status `passing`. Script exits 1. Assert TC status `failing`. Product has no knowledge of what the script does internally.