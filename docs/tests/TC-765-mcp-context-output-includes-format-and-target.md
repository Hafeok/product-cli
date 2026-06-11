---
id: TC-765
title: mcp-context-output-includes-format-and-target
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_765_mcp_context_output_includes_format_and_target
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `mcp-context-output-includes-format-and-target`

**Given** a connected MCP client calling `product_context`,
**When** the response is returned,
**Then** the JSON envelope contains exactly the keys `format`, `target`, `content`, `token_count_approx`, `exceeded_target_max`, `exceeded_hard_max` (in addition to whatever metadata existed pre-FT-063).

`exceeded_target_max` and `exceeded_hard_max` are booleans derived from the template's `[token_budget]` and the bundle's approximate token count. The bundle is never truncated; the booleans only flag the condition.

## Validates

- FT-063 — Per-Model Context Bundle Templates (MCP response shape)
- ADR-020 — MCP Server — Dual Transport (stdio and HTTP)
- ADR-049 — Per-Model Context Bundle Templates as Data Files