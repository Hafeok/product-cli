---
id: TC-099
title: mcp_stdio_tool_call
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "mcp_003_stdio_feature_list"
---

spawn `product mcp` as a subprocess. Send a valid JSON-RPC tool call over stdin. Assert the response on stdout matches the expected MCP tool result format.