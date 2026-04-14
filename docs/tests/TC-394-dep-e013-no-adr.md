---
id: TC-394
title: dep_e013_no_adr
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-030
phase: 1
---

DEP-005 has no `adrs` links. Run `product graph check`. Assert exit code 1 and E013 naming DEP-005 with the message "every dependency requires a governing decision."