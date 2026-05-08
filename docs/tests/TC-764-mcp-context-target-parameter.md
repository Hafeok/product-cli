---
id: TC-764
title: mcp-context-target-parameter
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_764_mcp_context_target_parameter
last-run: 2026-05-08T12:14:59.128626357+00:00
last-run-duration: 0.4s
---

## Scenario — `mcp-context-target-parameter`

**Given** a connected MCP client,
**When** the client calls `product_context` with `{ "feature_id": "FT-XXX", "depth": 2, "target": "claude-opus" }`,
**Then** the JSON response carries the rendered XML bundle in `content`, with `format = "xml"` and `target = "claude-opus"`.

Calling without a `target` parameter falls back to the configured default.

## Validates

- FT-063 — Per-Model Context Bundle Templates (MCP tool surface)
- ADR-020 — MCP Server — Dual Transport (stdio and HTTP)
- ADR-049 — Per-Model Context Bundle Templates as Data Files