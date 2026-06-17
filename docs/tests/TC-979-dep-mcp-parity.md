---
id: TC-979
title: dependency MCP parity via call_tool
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
runner-args: dep_list_show_features_via_call_tool
---

## Scenario — dependencies are inspectable over MCP

**Given** a repo with a dependency `DEP-001` (oxigraph) used by `FT-001`,
**When** the caller invokes `product_dep_{list,show,features}` via `call_tool`,
**Then** list includes `DEP-001`, show returns its title `oxigraph`, and features
includes `FT-001`.

## Validates

- FT-129 — CLI↔MCP parity for archetype, cell, how, work-unit, and dep — PENDING_MCP cleared
- ADR-070 — The remaining families expose read MCP tools; PENDING_MCP is emptied
