---
id: TC-417
title: product_agent_context MCP tool returns AGENT.md content
type: contract
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_417_product_agent_context_mcp_tool_returns_agent_md_content"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product agent-init` to generate `AGENT.md`. Call `product_agent_context` MCP read tool. Assert the response content matches the content of the generated `AGENT.md` file on disk.