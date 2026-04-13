---
id: TC-090
title: gap_check_changed_scoping
type: scenario
status: unimplemented
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

modify ADR-002 in git. Run `product gap check --changed`. Assert only ADR-002 and its 1-hop neighbours are analysed (not ADR-007 which shares no features).