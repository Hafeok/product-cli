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
runner-args: "tc_099_mcp_stdio_tool_call"
last-run: 2026-04-18T10:41:43.286383101+00:00
last-run-duration: 0.1s
---

spawn `product mcp` as a subprocess. Send a valid JSON-RPC tool call over stdin. Assert the response on stdout matches the expected MCP tool result format.