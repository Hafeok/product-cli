---
id: TC-982
title: domain MCP new rejects non-conformant
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
runner-args: tc_982_domain_mcp_new_rejects_non_conformant
---

## Scenario — the MCP create validates in-loop

**Given** the `product mcp --write` server with a bounded context created,
**When** the client calls `product_domain_new` for an event whose `changes`
target is not a real entity,
**Then** the process exits 0 and the response on stdout carries `ok: false`
with a violation naming the framework section `§3.2`.

## Validates

- FT-119 — product_domain_* MCP tools — CLI↔MCP parity for the What graph
- ADR-087 — The domain (What) graph is exposed as product_domain_* MCP tools
