---
id: TC-106
title: mcp_tool_registry_shared
type: contract
status: passing
validates:
  features:
  - FT-021
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_165_ft_021_mcp_server_stdio_and_http_pass"
last-run: 2026-04-18T10:41:43.286383101+00:00
last-run-duration: 0.3s
---

assert that calling `product_context` via stdio and via HTTP on the same repository produces identical output.