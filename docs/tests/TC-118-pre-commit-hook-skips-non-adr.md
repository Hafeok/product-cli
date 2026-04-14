---
id: TC-118
title: pre_commit_hook_skips_non_adr
type: scenario
status: passing
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_118_pre_commit_hook_skips_non_adr"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

stage a feature file. Assert the hook does not run `adr review`.