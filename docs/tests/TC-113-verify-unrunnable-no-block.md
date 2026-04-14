---
id: TC-113
title: verify_unimplemented_blocks
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_113_verify_unimplemented_blocks
last-run: 2026-04-14T18:03:59.846825760+00:00
last-run-duration: 0.2s
---

All TCs have no `runner` field. Assert feature goes to in-progress (unimplemented blocks completion). Distinct from `unrunnable` which is an explicit acknowledgement that does not block.