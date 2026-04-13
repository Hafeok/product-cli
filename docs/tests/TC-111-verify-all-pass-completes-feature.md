---
id: TC-111
title: verify_all_pass_completes_feature
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_111_verify_all_pass_completes_feature
last-run: 2026-04-13T14:07:16.920985096+00:00
---

all TCs configured with passing test runners. Run `product verify FT-001`. Assert all TCs become `passing` and feature becomes `complete`.