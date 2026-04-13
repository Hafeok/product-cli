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
last-run: 2026-04-13T14:07:16.920985096+00:00
---

run verify. Assert `checklist.md` is updated to reflect new TC statuses.