---
id: TC-384
title: dep_governs_edge
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_384_dep_governs_edge"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

ADR links `governs: [DEP-001]`. Assert graph contains both directions.