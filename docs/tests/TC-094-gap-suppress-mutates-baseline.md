---
id: TC-094
title: gap_suppress_mutates_baseline
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_094_gap_suppress_mutates_baseline"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

run `product gap suppress GAP-ADR002-G001-a3f9 --reason "deferred"`. Assert `gaps.json` contains the suppression with the reason, timestamp, and current commit hash.