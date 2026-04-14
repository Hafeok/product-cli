---
id: TC-395
title: dep_gap_g008
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_395_dep_gap_g008"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

feature uses DEP-005. No ADR has `governs: [DEP-005]`. Run `product gap check FT-007`. Assert G008 finding.