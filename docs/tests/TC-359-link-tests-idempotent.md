---
id: TC-359
title: link_tests_idempotent
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

run `product migrate link-tests` twice. Assert file content identical after both runs. Assert second run reports "0 new links."