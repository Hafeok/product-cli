---
id: TC-099
title: mcp_stdio_tool_call
type: contract
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_099_mcp_stdio_tool_call"
last-run: 2026-04-30T09:23:08.718018813+00:00
last-run-duration: 0.2s
---

spawn `product mcp` as a subprocess. Send a valid JSON-RPC tool call over stdin. Assert the response on stdout matches the expected MCP tool result format.