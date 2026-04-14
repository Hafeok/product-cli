---
id: TC-237
title: status_shows_phase_gate
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_237_status_shows_phase_gate"
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
last-run: 2026-04-14T15:03:33.506444091+00:00
---

run `product status`. Assert each phase shows its gate state: `[OPEN]`, `[LOCKED]`. Assert LOCKED phases name the failing exit-criteria TCs.