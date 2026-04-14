---
id: TC-340
title: w010_unacknowledged_cross_cutting
type: scenario
status: unimplemented
validates:
  features: 
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
---

ADR-013 is cross-cutting. FT-009 neither links nor acknowledges it. Run `product graph check`. Assert W010 naming FT-009 and ADR-013.