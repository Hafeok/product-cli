---
id: TC-304
title: verify_one_fail_in_progress
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_304_verify_one_fail_in_progress
last-run: 2026-04-18T10:41:51.294040135+00:00
last-run-duration: 0.2s
---

one TC fails. Assert feature stays `in-progress`.