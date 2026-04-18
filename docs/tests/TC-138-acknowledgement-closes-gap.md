---
id: TC-138
title: acknowledgement_closes_gap
type: scenario
status: passing
validates:
  features:
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
runner: cargo-test
runner-args: "tc_138_acknowledgement_closes_gap"
last-run: 2026-04-18T10:41:39.917241637+00:00
last-run-duration: 0.2s
---

FT-009 has `domains-acknowledged: { security: "no trust boundaries" }`. Assert W011 does not fire for FT-009's security domain.