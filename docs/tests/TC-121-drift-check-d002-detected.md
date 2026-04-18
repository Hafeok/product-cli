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
last-run: 2026-04-18T10:41:56.996985101+00:00
last-run-duration: 0.2s
---

fixture with ADR saying "use openraft", source file using a custom Raft struct. Assert D002 finding.