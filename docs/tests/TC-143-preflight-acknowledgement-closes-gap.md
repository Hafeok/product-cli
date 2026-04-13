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
---

run `product feature acknowledge FT-009 --domain security --reason "no trust boundaries"`. Re-run preflight. Assert security gap closed. Assert exit 0.