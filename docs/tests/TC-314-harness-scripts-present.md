---
id: TC-314
title: harness_scripts_present
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_314_harness_scripts_present
last-run: 2026-04-14T18:03:59.846825760+00:00
last-run-duration: 0.2s
---

assert `scripts/harness/implement.sh` and `scripts/harness/author.sh` exist and are executable.