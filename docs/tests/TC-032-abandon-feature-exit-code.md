---
id: TC-032
title: abandon_feature_exit_code
type: exit-criteria
status: passing
validates:
  features:
  - FT-018
  adrs:
  - ADR-010
phase: 1
runner: cargo-test
runner-args: "tc_032_abandon_feature_exit_code"
---

after abandoning a feature with linked tests, run `product graph check`. Assert exit code 2 (warning) not 1 (error).