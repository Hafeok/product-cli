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
runner: cargo-test
runner-args: "tc_070_tmp_cleanup_on_startup"
---

create leftover `.product-tmp.*` files. Run `product feature list` (read-only). Assert the tmp files are deleted on startup.