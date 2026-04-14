---
id: TC-388
title: dep_preflight_check_fails
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_388_dep_preflight_check_fails"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

DEP-005 availability check exits 1. Assert preflight report names DEP-005 as unavailable. Assert exit code 2 (warning, not error).