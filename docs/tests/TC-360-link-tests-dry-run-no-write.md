---
id: TC-360
title: link_tests_dry_run_no_write
type: scenario
status: passing
validates:
  features: 
  - FT-030
  adrs:
  - ADR-027
phase: 1
runner: cargo-test
runner-args: "tc_360_link_tests_dry_run_no_write"
last-run: 2026-04-14T18:29:03.925615608+00:00
last-run-duration: 0.2s
---

run `product migrate link-tests --dry-run`. Assert zero files modified. Assert stdout contains the inference plan.