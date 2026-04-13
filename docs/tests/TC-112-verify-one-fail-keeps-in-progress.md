---
id: TC-112
title: verify_one_fail_keeps_in_progress
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_112_verify_one_fail_keeps_in_progress
last-run: 2026-04-13T14:07:16.920985096+00:00
---

one TC fails. Assert feature stays `in-progress`.