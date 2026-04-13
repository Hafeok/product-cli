---
id: TC-100
title: mcp_http_tool_call
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_100_mcp_http_tool_call"
---

start `product mcp --http --port 17777 --token test`. Send an HTTP POST to `http://localhost:17777/mcp`. Assert 200 response with correct tool result.