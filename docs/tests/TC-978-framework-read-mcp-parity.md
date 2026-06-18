---
id: TC-978
title: framework read MCP parity via call_tool
type: scenario
status: passing
validates:
  features:
  - FT-129
  adrs:
  - ADR-070
phase: 6
observes:
- stdout
runner: cargo-test
runner-args: how_show_validate_export
---

## Scenario — how/archetype/cell/work-unit are inspectable over MCP

**Given** a repo with `.product/how-contract.yaml`, `cell.yaml`, `work-unit.yaml`,
and an archetype directory,
**When** the caller invokes `product_how_{show,validate,export}` (and the
archetype/cell/work-unit read tools) via `call_tool`,
**Then** `how_show` reports ≥1 principle, `how_validate` returns `ok: true`, and
`how_export` returns Turtle containing the `pf:` prefix.

## Validates

- FT-129 — CLI↔MCP parity for archetype, cell, how, work-unit, and dep — PENDING_MCP cleared
- ADR-070 — The remaining families expose read MCP tools; PENDING_MCP is emptied
