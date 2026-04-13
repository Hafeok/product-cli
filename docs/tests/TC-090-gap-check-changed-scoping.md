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
---

modify ADR-002 in git. Run `product gap check --changed`. Assert only ADR-002 and its 1-hop neighbours are analysed (not ADR-007 which shares no features).