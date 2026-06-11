---
id: TC-769
title: mcp-product-context-uses-id-parameter
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_769_mcp_product_context_uses_id_parameter
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `mcp-product-context-uses-id-parameter`

**Given** the MCP `product_context` tool is registered,
**When** a client calls `tools/list` and `tools/call` with arguments
`{"id": "FT-XXX", "target": "claude-opus"}`,
**Then** the tool's `inputSchema.properties` advertises an `id` property
(not `feature_id`), and the call returns the templated bundle envelope
`{format, target, content, token_count_approx, exceeded_target_max,
exceeded_hard_max}`.

The drift this test locks down was found during the FT-063 e2e shake-out:
the original FT-063 PRD example showed `{"feature_id": "FT-009", ...}` in
the MCP input block, but every other MCP read tool (`product_feature_show`,
`product_adr_show`, `product_test_show`, ...) uses `id` as the canonical
property name. The implementation correctly reads `id`; the PRD example
was wrong. This test pins `id` as the canonical name so the convention
cannot drift.

## Validates

- FT-063 — Per-Model Context Bundle Templates (MCP input shape)
- ADR-049 — Per-Model Context Bundle Templates as Data Files