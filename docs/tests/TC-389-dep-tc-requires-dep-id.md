---
id: TC-389
title: dep_tc_requires_dep_id
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_389_dep_tc_requires_dep_id"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

TC declares `requires: [DEP-005]`. Product resolves to DEP-005's availability check. Assert the resolved check command matches DEP-005 `availability-check`.