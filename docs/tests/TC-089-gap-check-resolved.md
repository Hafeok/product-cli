---
id: TC-089
title: gap_check_resolved
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_089_gap_check_resolved"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

suppress a gap, then fix it (add the missing TC). Run analysis. Assert the gap no longer appears in findings. Assert `gaps.json` resolved list is updated.