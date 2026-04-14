---
id: TC-240
title: context_depth_dedup
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

two paths from FT-001 to ADR-002 (via direct link and via depends-on chain). Assert ADR-002 appears exactly once in the bundle.