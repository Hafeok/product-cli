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
---

assert that calling `product_context` via stdio and via HTTP on the same repository produces identical output.