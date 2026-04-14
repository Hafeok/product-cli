---
id: TC-086
title: gap_check_single_adr
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_086_gap_check_single_adr"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

run `product gap check ADR-001` against a fixture where ADR-001 has a testable claim with no linked TC. Assert exit code 1 and a G001 finding in stdout JSON.