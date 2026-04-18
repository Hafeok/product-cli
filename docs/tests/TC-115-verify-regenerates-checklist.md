---
id: TC-115
title: verify_regenerates_checklist
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_115_verify_regenerates_checklist
last-run: 2026-04-18T10:41:51.294040135+00:00
last-run-duration: 0.2s
---

run verify. Assert `checklist.md` is updated to reflect new TC statuses.