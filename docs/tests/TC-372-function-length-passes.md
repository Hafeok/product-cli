---
id: TC-372
title: function_length_passes
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_372_function_length_passes"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

all functions under 30 statement lines. Assert exit 0.