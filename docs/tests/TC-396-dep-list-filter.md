---
id: TC-396
title: dep_list_filter
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_396_dep_list_filter"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

run `product dep list --type service`. Assert only service-type dependencies returned.