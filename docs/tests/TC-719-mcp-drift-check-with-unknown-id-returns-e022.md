---
id: TC-719
title: mcp drift check with unknown id returns E022
type: scenario
status: passing
validates:
  features:
  - FT-059
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_719_mcp_drift_check_unknown_id_returns_e022
last-run: 2026-05-06T13:31:35.056793027+00:00
last-run-duration: 0.2s
---

## Given

A temp Product repository with a known set of ADR and feature IDs. The string `"ADR-9999"` does not exist in the graph.

`product` is launched in MCP stdio mode against the temp repo.

## When

The test sends a `tools/call` JSON-RPC request for `product_drift_check` with `{ "id": "ADR-9999" }`.

## Then

- The JSON-RPC response carries an `error` object (or the tool result is an error envelope, depending on the registry's existing convention — the test asserts whichever the registry already uses for `product_adr_show` on an unknown ID).
- The error payload identifies the code as `E022` and the message contains `"health-check-id-not-found"` and the literal `"ADR-9999"`.
- No file under the temp repo is modified after the call (mtime on `drift.json` baseline is unchanged).