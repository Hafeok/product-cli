---
id: TC-722
title: mcp preflight with unknown id returns E022
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_722_mcp_preflight_unknown_id_returns_e022
---

## Given

A temp Product repository where `"FT-9999"` is not present in `graph.features`.

`product` is launched in MCP stdio mode against the temp repo.

## When

The test sends a `tools/call` JSON-RPC request for `product_preflight` with `{ "id": "FT-9999" }`.

## Then

- The response carries an error result.
- The error code is `E022` and the message contains `"health-check-id-not-found"` and the literal `"FT-9999"`.
- No feature, ADR, or TC files are written.
