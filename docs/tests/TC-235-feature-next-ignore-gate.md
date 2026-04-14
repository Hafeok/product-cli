---
id: TC-235
title: feature_next_ignore_gate
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

phase-1 exit criteria failing. Run `product feature next --ignore-phase-gate`. Assert a phase-2 feature is returned. Assert a warning is emitted to stderr.