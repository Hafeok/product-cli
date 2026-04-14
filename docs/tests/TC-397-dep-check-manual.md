---
id: TC-397
title: dep_check_manual
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_397_dep_check_manual"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

run `product dep check DEP-005` with availability check that exits 0. Assert output shows check passed. With exit 1: assert shows failed.