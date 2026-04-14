---
id: TC-391
title: dep_bom_output
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_391_dep_bom_output"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

run `product dep bom`. Assert output groups by type, lists all active dependencies. Assert `--format json` produces valid JSON.