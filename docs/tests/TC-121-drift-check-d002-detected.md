---
id: TC-121
title: drift_check_d002_detected
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-023
phase: 1
runner: cargo-test
runner-args: "tc_121_drift_check_d002_detected"
last-run: 2026-04-13T14:27:30.366814571+00:00
---

fixture with ADR saying "use openraft", source file using a custom Raft struct. Assert D002 finding.