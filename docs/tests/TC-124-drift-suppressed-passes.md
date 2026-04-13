---
id: TC-124
title: drift_suppressed_passes
type: scenario
status: passing
validates:
  features:
  - FT-028
  adrs:
  - ADR-023
phase: 1
runner: cargo-test
runner-args: "tc_124_drift_suppressed_passes"
last-run: 2026-04-13T14:27:30.366814571+00:00
---

suppress a D002 finding. Run drift check. Assert exit 0.