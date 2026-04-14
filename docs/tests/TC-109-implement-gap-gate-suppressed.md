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
last-run: 2026-04-14T18:03:59.846825760+00:00
last-run-duration: 0.2s
---

same feature with the gap suppressed. Assert pipeline proceeds past gap gate.