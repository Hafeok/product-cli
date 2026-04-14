---
id: TC-393
title: dep_w013_deprecated
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_393_dep_w013_deprecated"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

DEP-005 status `deprecated`. Feature FT-007 uses DEP-005. Run `product graph check`. Assert W013 naming FT-007 and DEP-005.