---
id: TC-363
title: feature_link_interactive_confirm
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

run `product feature link FT-009 --adr ADR-021`. Assert interactive prompt shows inferred TC links. On confirmation, assert TC links applied atomically with the ADR link.