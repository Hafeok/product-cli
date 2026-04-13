---
id: TC-091
title: gap_check_model_error_exits_2
type: exit-criteria
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_091_gap_check_model_error_exits_2"
---

inject a network failure for the model call. Assert exit code 2 (warning), not 1 (new gaps). Assert error appears on stderr.