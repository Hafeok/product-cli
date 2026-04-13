---
id: TC-044
title: feature_next_uses_topo
type: scenario
status: passing
validates:
  features:
  - FT-011
  - FT-016
  adrs:
  - ADR-012
phase: 1
---

FT-001 complete, FT-002 depends-on FT-001 (in-progress), FT-003 no dependencies (planned). Assert `product feature next` returns FT-002, not FT-003.