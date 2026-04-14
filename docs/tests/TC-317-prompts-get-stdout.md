---
id: TC-317
title: prompts_get_stdout
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_317_prompts_get_stdout"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

run `product prompts get author-feature`. Assert stdout contains the prompt content. Assert stderr is empty.