---
id: TC-109
title: implement_gap_gate_suppressed
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_109_implement_gap_gate_suppressed
last-run: 2026-04-30T09:23:18.004925059+00:00
last-run-duration: 0.3s
---

same feature with the gap suppressed. Assert pipeline proceeds past gap gate.