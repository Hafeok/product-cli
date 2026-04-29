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
last-run: 2026-04-29T03:12:46.161410171+00:00
last-run-duration: 0.2s
---

feature with G001 gap unsuppressed. Assert `product implement` exits 1 and prints E009 with the gap details.