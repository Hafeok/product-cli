---
id: TC-238
title: status_phase_detail
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_238_status_phase_detail"
validates:
  features: 
  - FT-006
  - FT-011
  - FT-014
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
last-run: 2026-04-14T15:02:16.595537282+00:00
---

run `product status --phase 1`. Assert output lists all exit-criteria TCs for phase 1 with their individual pass/fail status.