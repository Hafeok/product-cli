---
id: TC-088
title: gap_check_suppressed
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "suppression_works"
---

add a suppression for a known gap to `gaps.json`. Run analysis. Assert exit code 0. Assert the finding appears in output with `"suppressed": true`.