---
id: TC-416
title: product_schema MCP tool returns schema for artifact type
type: scenario
status: unimplemented
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
---

## Description

Call `product_schema` MCP tool with argument `feature`. Assert the response content matches `product schema feature` CLI output. Repeat for `adr`, `test`, `dep`. Assert `--all` variant also works via MCP.
