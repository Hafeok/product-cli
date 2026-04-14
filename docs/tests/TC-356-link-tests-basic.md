---
id: TC-356
title: link_tests_basic
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

FT-001 links ADR-002. TC-002 validates ADR-002. Run `product migrate link-tests`. Assert TC-002 gains `validates.features: [FT-001]`. Assert FT-001 gains `tests: [TC-002]`.