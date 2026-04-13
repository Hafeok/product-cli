---
id: TC-033
title: abandon_feature_stdout
type: scenario
status: passing
validates:
  features:
  - FT-018
  adrs:
  - ADR-010
phase: 1
runner: cargo-test
runner-args: "tc_033_abandon_feature_stdout"
---

assert the abandonment command prints the list of test criteria that were auto-orphaned.