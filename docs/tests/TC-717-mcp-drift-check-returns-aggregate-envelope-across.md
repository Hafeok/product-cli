---
id: TC-717
title: mcp drift check returns aggregate envelope across all ADRs
type: scenario
status: passing
validates:
  features:
  - FT-059
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_717_mcp_drift_check_aggregate_envelope
last-run: 2026-05-06T13:31:35.056793027+00:00
last-run-duration: 0.2s
---

## Given

A temp Product repository populated via `product request apply` with at least three accepted ADRs that declare `source-files`, and at least one `src/` file that intentionally drifts from one ADR's signature pattern.

The compiled `product` binary is launched in MCP stdio mode against the temp repo.

## When

The test sends a `tools/call` JSON-RPC request for `product_drift_check` with no arguments.

## Then

- The JSON-RPC response is a `success` result (no `error` field).
- The `content[0].text` payload parses as JSON.
- The parsed object has top-level keys `status`, `checked`, `findings`, `summary`.
- `status` is one of `"clean" | "warnings" | "findings"`.
- `summary.high + summary.medium + summary.low` equals `findings.iter().filter(|f| !f.suppressed).count()`.
- The number of distinct `adr_id` values across `findings` is at least one (the seeded drift case).
- The same temp repo, when `product drift check --format json` is invoked via the CLI, produces a findings array whose `id` set equals the MCP tool's `findings[*].id` set (parity check).