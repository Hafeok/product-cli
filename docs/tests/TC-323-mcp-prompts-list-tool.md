---
id: TC-323
title: mcp_prompts_list_tool
type: scenario
status: passing
validates:
  features: 
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_323_mcp_prompts_list_tool"
last-run: 2026-04-14T17:42:46.235479401+00:00
---

call `product_prompts_list` via MCP. Assert JSON response lists available prompts.