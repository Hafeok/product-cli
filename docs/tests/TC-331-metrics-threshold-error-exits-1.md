---
id: TC-331
title: metrics_threshold_error_exits_1
type: exit-criteria
status: unimplemented
validates:
  features: 
  - FT-028
  adrs:
  - ADR-024
phase: 1
---

set `spec_coverage` threshold, configure a repo below it. Run `product metrics threshold`. Assert exit code 1.