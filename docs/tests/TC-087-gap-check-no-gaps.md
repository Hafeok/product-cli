---
id: TC-087
title: gap_check_no_gaps
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_087_gap_check_no_gaps"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

run `product gap check ADR-001` against a fixture with full TC coverage. Assert exit code 0 and an empty findings array.