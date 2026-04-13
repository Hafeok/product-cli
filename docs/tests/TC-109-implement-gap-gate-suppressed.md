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
last-run: 2026-04-13T14:07:16.920985096+00:00
---

same feature with the gap suppressed. Assert pipeline proceeds past gap gate.