---
id: TC-324
title: mcp_prompts_get_tool
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_324_mcp_prompts_get_tool"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

call `product_prompts_get` with `name: "author-feature"`. Assert response contains prompt content.