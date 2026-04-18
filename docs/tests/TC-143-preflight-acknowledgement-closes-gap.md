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
last-run: 2026-04-18T10:41:54.811678685+00:00
last-run-duration: 0.2s
---

run `product feature acknowledge FT-009 --domain security --reason "no trust boundaries"`. Re-run preflight. Assert security gap closed. Assert exit 0.