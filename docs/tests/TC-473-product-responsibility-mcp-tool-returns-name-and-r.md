---
id: TC-473
title: product_responsibility MCP tool returns name and responsibility
type: contract
status: passing
validates:
  features:
  - FT-039
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: "tc_473_product_responsibility_mcp_tool_returns_name_and_responsibility"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.4s
---

**Given** a repository with `[product].responsibility` set in product.toml
**When** the `product_responsibility` MCP tool is called with no arguments
**Then** the response contains `name` and `responsibility` fields matching the configured values

**Given** a repository without `[product].responsibility` in product.toml
**When** the `product_responsibility` MCP tool is called
**Then** the response returns an error indicating the responsibility field is not configured