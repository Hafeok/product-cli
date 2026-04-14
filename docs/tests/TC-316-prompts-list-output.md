---
id: TC-316
title: prompts_list_output
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_316_prompts_list_output"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

run `product prompts list`. Assert output lists all prompt files with version numbers.