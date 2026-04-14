---
id: TC-090
title: gap_check_changed_scoping
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_090_gap_check_changed_scoping"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

modify ADR-002 in git. Run `product gap check --changed`. Assert only ADR-002 and its 1-hop neighbours are analysed (not ADR-007 which shares no features).