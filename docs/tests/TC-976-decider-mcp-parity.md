---
id: TC-976
title: decider MCP parity via call_tool
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
runner-args: derive_then_validate_and_list_via_call_tool
---

## Scenario — the decider family is drivable over MCP

**Given** a registry over a repo with a captured What graph containing an
aggregate `Order`,
**When** the caller invokes `product_decider_derive` then
`product_decider_{list,validate,simulate}` via `call_tool`,
**Then** derive returns `ok` with id `Order-decider`, list includes it, validate
returns `ok: true`, and simulate reports `sound_and_complete: false` (no logic
authored yet).

## Validates

- FT-128 — CLI↔MCP parity for the decider, slice, deliverable, and release families
- ADR-069 — The framework families expose MCP tools mirroring their CLI subcommands
