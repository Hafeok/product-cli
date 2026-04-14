---
id: TC-360
title: link_tests_dry_run_no_write
type: scenario
status: unimplemented
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
---

run `product migrate link-tests --dry-run`. Assert zero files modified. Assert stdout contains the inference plan.