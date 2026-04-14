---
id: TC-236
title: feature_next_gate_partial
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

phase 1 has 4 exit-criteria TCs: 3 passing, 1 failing. Assert phase gate is NOT satisfied (all must pass). Assert stderr names only the failing TC.