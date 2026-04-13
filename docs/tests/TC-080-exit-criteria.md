---
id: TC-080
title: exit_criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_080_exit_criteria"
---

Migration extracts exit-criteria test type from ADR subsections titled "### Exit criteria". Bullets under that heading produce test files with `type: exit-criteria`.