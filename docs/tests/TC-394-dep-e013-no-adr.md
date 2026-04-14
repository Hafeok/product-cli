---
id: TC-394
title: dep_e013_no_adr
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_394_dep_e013_no_adr"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

DEP-005 has no `adrs` links. Run `product graph check`. Assert exit code 1 and E013 naming DEP-005 with the message "every dependency requires a governing decision."