---
id: TC-236
title: feature_next_gate_partial
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_236_feature_next_gate_partial"
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

phase 1 has 4 exit-criteria TCs: 3 passing, 1 failing. Assert phase gate is NOT satisfied (all must pass). Assert stderr names only the failing TC.