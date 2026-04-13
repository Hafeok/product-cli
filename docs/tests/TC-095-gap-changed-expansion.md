---
id: TC-095
title: gap_changed_expansion
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_095_gap_changed_expansion"
---

fixture: ADR-002 and ADR-005 share feature FT-001. Modify ADR-002. Run `--changed`. Assert ADR-005 is included in the analysis set.