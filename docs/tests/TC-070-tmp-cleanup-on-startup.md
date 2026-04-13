---
id: TC-070
title: tmp_cleanup_on_startup
type: scenario
status: passing
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
---

create leftover `.product-tmp.*` files. Run `product feature list` (read-only). Assert the tmp files are deleted on startup.