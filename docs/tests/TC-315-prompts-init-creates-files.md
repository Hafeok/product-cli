---
id: TC-315
title: prompts_init_creates_files
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_315_prompts_init_creates_files"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

run `product prompts init` on a repo with no `benchmarks/prompts/`. Assert all default prompt files are created.