---
id: TC-084
title: validates.adrs
type: scenario
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_084_validates_adrs"
---

Test criteria extracted from an ADR have validates.adrs containing the source ADR ID. Each test bullet under ADR-005's test section produces a TC file with `validates.adrs: [ADR-005]`.