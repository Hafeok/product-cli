---
id: TC-108
title: implement_gap_gate_blocks
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_108_implement_gap_gate_blocks
last-run: 2026-04-13T14:07:16.920985096+00:00
---

feature with G001 gap unsuppressed. Assert `product implement` exits 1 and prints E009 with the gap details.