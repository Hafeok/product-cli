---
id: TC-237
title: status_shows_phase_gate
type: scenario
status: unimplemented
validates:
  features: 
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-012
phase: 1
---

run `product status`. Assert each phase shows its gate state: `[OPEN]`, `[LOCKED]`. Assert LOCKED phases name the failing exit-criteria TCs.