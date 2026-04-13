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
---

FT-009 has `domains-acknowledged: { security: "no trust boundaries" }`. Assert W011 does not fire for FT-009's security domain.