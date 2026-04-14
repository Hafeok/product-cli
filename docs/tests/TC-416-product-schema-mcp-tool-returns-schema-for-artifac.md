---
id: TC-416
title: product_schema MCP tool returns schema for artifact type
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_416_product_schema_mcp_tool_returns_schema_for_artifact_type"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Call `product_schema` MCP tool with argument `feature`. Assert the response content matches `product schema feature` CLI output. Repeat for `adr`, `test`, `dep`. Assert `--all` variant also works via MCP.