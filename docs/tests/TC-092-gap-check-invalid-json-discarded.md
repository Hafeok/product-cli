---
id: TC-092
title: gap_check_invalid_json_discarded
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_092_gap_check_invalid_json_discarded"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

inject a model response with one valid finding and one malformed finding. Assert the valid finding is in output. Assert the malformed finding is logged to stderr and discarded.