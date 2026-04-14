---
id: TC-358
title: link_tests_cross_cutting_excluded
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

ADR-001 is cross-cutting. TC-001 validates ADR-001. All features link ADR-001. Run `link-tests`. Assert TC-001.validates.features remains empty.