---
id: TC-413
title: AGENT.md contains MCP tool usage guide
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_413_agent_md_contains_mcp_tool_usage_guide"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product agent-init`. Assert `AGENT.md` contains a "Key MCP Tools" section with a table listing at least: `product_context`, `product_schema`, `product_graph_central`, `product_preflight`, `product_gap_check`, `product_agent_context`.