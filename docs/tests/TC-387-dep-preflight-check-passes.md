---
id: TC-387
title: dep_preflight_check_passes
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_387_dep_preflight_check_passes"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

DEP-005 has `availability-check` that exits 0. Run `product preflight FT-007`. Assert DEP-005 shows as available.