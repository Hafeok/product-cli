---
id: TC-265
title: tmp_cleanup_on_startup
type: scenario
status: unimplemented
validates:
  features: 
  - FT-004
  - FT-005
  adrs:
  - ADR-015
phase: 1
---

create leftover `.product-tmp.*` files. Run `product feature list` (read-only). Assert the tmp files are deleted on startup.