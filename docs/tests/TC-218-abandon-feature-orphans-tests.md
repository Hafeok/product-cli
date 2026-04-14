---
id: TC-218
title: abandon_feature_orphans_tests
type: scenario
status: unimplemented
validates:
  features: 
  - FT-018
  adrs:
  - ADR-010
phase: 1
---

create FT-001 linked to TC-001 and TC-002. Set FT-001 to `abandoned`. Assert TC-001 and TC-002 have FT-001 removed from their `validates.features`. Assert both tests appear in `product test untested`.