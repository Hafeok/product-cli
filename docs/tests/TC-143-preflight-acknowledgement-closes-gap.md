---
id: TC-143
title: preflight_acknowledgement_closes_gap
type: scenario
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_143_preflight_acknowledgement_closes_gap"
last-run: 2026-04-13T14:12:26.396687298+00:00
---

run `product feature acknowledge FT-009 --domain security --reason "no trust boundaries"`. Re-run preflight. Assert security gap closed. Assert exit 0.