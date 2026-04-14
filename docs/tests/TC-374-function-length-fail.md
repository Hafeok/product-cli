---
id: TC-374
title: function_length_fail
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_374_function_length_fail"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

one function with 45 statement lines. Assert exit 1 with file path and line number.