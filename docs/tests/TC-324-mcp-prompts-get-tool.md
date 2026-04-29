---
id: TC-324
title: mcp_prompts_get_tool
type: contract
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_324_mcp_prompts_get_tool"
last-run: 2026-04-29T03:12:43.749153090+00:00
last-run-duration: 0.2s
---

call `product_prompts_get` with `name: "author-feature"`. Assert response contains prompt content.