---
id: TC-473
title: product_responsibility MCP tool returns name and responsibility
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Given** a repository with `[product].responsibility` set in product.toml
**When** the `product_responsibility` MCP tool is called with no arguments
**Then** the response contains `name` and `responsibility` fields matching the configured values

**Given** a repository without `[product].responsibility` in product.toml
**When** the `product_responsibility` MCP tool is called
**Then** the response returns an error indicating the responsibility field is not configured