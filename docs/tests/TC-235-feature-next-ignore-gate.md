---
id: TC-235
title: feature_next_ignore_gate
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_235_feature_next_ignore_gate"
validates:
  features: 
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
last-run: 2026-04-14T14:04:19.495078770+00:00
---

phase-1 exit criteria failing. Run `product feature next --ignore-phase-gate`. Assert a phase-2 feature is returned. Assert a warning is emitted to stderr.