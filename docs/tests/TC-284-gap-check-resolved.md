---
id: TC-284
title: gap_check_resolved
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

suppress a gap, then fix it (add the missing TC). Run analysis. Assert the gap no longer appears in findings. Assert `gaps.json` resolved list is updated.