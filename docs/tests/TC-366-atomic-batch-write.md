---
id: TC-366
title: atomic_batch_write
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

inject a write failure midway through a multi-file inference batch. Assert all-or-nothing: either all files updated or none. Assert no partial state.