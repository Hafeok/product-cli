---
id: TC-034
title: abandon_feature_tests_preserved
type: scenario
status: passing
validates:
  features:
  - FT-018
  adrs:
  - ADR-010
phase: 1
runner: cargo-test
runner-args: "tc_034_abandon_feature_tests_preserved"
---

assert test criterion files are not deleted during abandonment, only their feature links are removed.