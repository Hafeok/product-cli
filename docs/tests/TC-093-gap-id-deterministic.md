---
id: TC-093
title: gap_id_deterministic
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_093_gap_id_deterministic"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

run gap analysis twice against identical repository state. Assert all high-severity findings have identical IDs between runs.