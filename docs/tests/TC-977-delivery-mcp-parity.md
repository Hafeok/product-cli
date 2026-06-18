---
id: TC-977
title: delivery MCP parity via call_tool
type: scenario
status: passing
validates:
  features:
  - FT-128
  adrs:
  - ADR-069
phase: 6
observes:
- stdout
runner: cargo-test
runner-args: delivery_chain_and_done_via_call_tool
---

## Scenario — the delivery families are drivable over MCP

**Given** a registry over a repo with a captured What graph,
**When** the caller invokes `product_slice_new` + `product_slice_context`,
`product_deliverable_{new,done,accept}`, and `product_release_{new,done}` via
`call_tool`,
**Then** the slice context bundle contains `PlaceOrder`, the deliverable is not
done until acceptance is recorded passing, and the release is `done` with
`closed: true`.

## Validates

- FT-128 — CLI↔MCP parity for the decider, slice, deliverable, and release families
- ADR-069 — The framework families expose MCP tools mirroring their CLI subcommands
