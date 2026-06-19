---
id: TC-981
title: domain MCP tools have CLI parity
type: scenario
status: passing
validates:
  features:
  - FT-119
  adrs:
  - ADR-087
phase: 6
observes:
- exit-code
- stdout
runner: cargo-test
runner-args: tc_981_domain_mcp_tools_have_cli_parity
---

## Scenario — the domain tools work through the MCP server

**Given** the `product mcp --write` server,
**When** the client sends, over stdin JSON-RPC, `product_domain_new` for a
context and an entity, then `product_domain_validate` and
`product_domain_list`,
**Then** the process exits 0; on stdout each create returns `ok: true`,
validate returns `conformant: true`, and list returns `count: 2` — the same
result the equivalent `product domain` CLI calls produce.

## Validates

- FT-119 — product_domain_* MCP tools — CLI↔MCP parity for the What graph
- ADR-087 — The domain (What) graph is exposed as product_domain_* MCP tools
