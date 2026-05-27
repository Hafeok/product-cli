---
id: TC-809
title: mcp_graph_check_surfaces_log_verification_finding
type: scenario
status: passing
validates:
  features:
  - FT-069
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_809_mcp_graph_check_surfaces_log_verification_finding
last-run: 2026-05-27T11:04:45.120555493+00:00
last-run-duration: 0.2s
---

## Scenario

When `[log].verify-on-check = true` and `requests.jsonl` has a
broken hash chain, the MCP `product_graph_check` tool propagates
the `verify_log` finding alongside the structural findings — exactly
as the CLI does.

### Given

A temp repository fixture with:

- `[log].verify-on-check = true` in `product.toml`.
- A `requests.jsonl` file at the configured `[paths].requests` whose
  hash chain is intentionally corrupted (e.g. a mutated `entry-hash`
  on the second entry).

### When

The MCP client invokes `product_graph_check` against the fixture.

### Then

- The returned JSON envelope contains the log-verification finding
  from `request_log::verify::verify_log` (matching code and message).
- The CLI `product graph check --format json` returns the same
  finding.
- The two envelopes are equal under the log-finding subset.