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
runner-args: "gap_id_deterministic"
---

run gap analysis twice against identical repository state. Assert all high-severity findings have identical IDs between runs.