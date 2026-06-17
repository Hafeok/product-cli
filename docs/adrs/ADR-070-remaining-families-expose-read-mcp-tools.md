---
id: ADR-070
title: The remaining families expose read MCP tools; PENDING_MCP is emptied
status: accepted
features:
- FT-129
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
source-files:
- product-mcp/src/framework_read_handlers.rs
- product-mcp/src/dep_handlers.rs
- product-mcp/src/tools/legacy_pf.rs
- product-mcp/src/registry.rs
- product-cli/tests/code_quality_tests.rs
---

## Context

After FT-128, `PENDING_MCP` still listed `archetype`, `cell`, `how`,
`work-unit`, `dep` — the pre-existing parity debt. Clearing it makes the whole
CLI surface drivable over MCP and removes the standing debt the FT-118 gate
exists to track.

## Decision

Expose read/inspection tools for the five families and empty `PENDING_MCP`:

- `archetype`/`cell`/`how`/`work-unit` read `.product/` artifacts via
  `product_core::pf` (`framework_read_handlers`, reusing `pf_mcp` loaders).
- `dep` queries the loaded knowledge graph (`dep_handlers`), since dependencies
  live in the legacy FT/ADR/TC graph, not the framework artifacts.
- All tools are `requires_write: false`. The write/scaffold/dispatch surfaces
  (`how add/set`, `cell dispatch`, the `*_init` scaffolds) are intentionally left
  CLI-only for now — they are authoring flows, not inspection, and can be added
  if agent-driven How authoring over MCP is wanted.

`PENDING_MCP` becomes empty; the parity gate (TC-980) now asserts every family
either has a tool or is explicitly `CLI_ONLY`, with no remaining debt.

## Rationale

- Read parity is the high-value, low-risk step: it lets an agent inspect every
  part of the graph (validate an archetype, export the How as Turtle, list deps)
  without exposing authoring flows that have large, branchy argument surfaces.
- Routing `dep` through the graph (not `.product/`) matches where the data lives,
  reusing the already-loaded graph in `call_tool`.
- Emptying `PENDING_MCP` (rather than leaving partial entries) makes the gate's
  state unambiguous: debt is zero.

## Rejected alternatives

- **Classify these as CLI_ONLY.** Rejected: inspection over MCP is genuinely
  useful for an agent; CLI_ONLY is for commands that should never have a tool.
- **Full write parity now.** Deferred: `how add/set` alone is a large branchy
  surface; read parity clears the debt and the writes can follow on demand.

## Test coverage

- TC-978 — framework read tools (how/archetype/cell/work-unit) via call_tool.
- TC-979 — dependency tools (list/show/features) via call_tool.
- TC-980 (parity gate) holds with an empty PENDING_MCP.
