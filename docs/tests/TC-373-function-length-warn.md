---
id: TC-373
title: function_length_warn
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_373_function_length_warn"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

one function with 35 statement lines. Assert exit 2.