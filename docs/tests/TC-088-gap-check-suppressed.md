---
id: TC-088
title: gap_check_suppressed
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_088_gap_check_suppressed"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

add a suppression for a known gap to `gaps.json`. Run analysis. Assert exit code 0. Assert the finding appears in output with `"suppressed": true`.