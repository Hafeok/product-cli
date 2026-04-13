---
id: TC-106
title: mcp_tool_registry_shared
type: scenario
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_165_ft_021_mcp_server_stdio_and_http_pass"
---

assert that calling `product_context` via stdio and via HTTP on the same repository produces identical output.