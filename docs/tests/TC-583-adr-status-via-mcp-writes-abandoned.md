---
id: TC-583
title: adr_status_via_mcp_writes_abandoned
type: scenario
status: unimplemented
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: tc_583_adr_status_via_mcp_writes_abandoned
---

## Session: adr_status_via_mcp_writes_abandoned

**Validates:** FT-046, ADR-020

### Given

A temp repository with ADR-099 in status `accepted` (sealed with a content-hash) and at least one feature still linking to ADR-099.

### When

An MCP client calls `product_adr_status` with `id: ADR-099, status: abandoned`.

### Then

- The call returns `{ id: "ADR-099", status: "abandoned", content-hash: <preserved> }`.
- The on-disk ADR-099 front-matter shows `status: abandoned`; `content-hash` is preserved (abandonment does not unseal — the decision record still exists).
- The linked features are not modified by this call; their `adrs` arrays still reference ADR-099.
- `product graph check` may now emit a W-class warning `"feature FT-XXX links abandoned ADR ADR-099"` — this is advisory and not a failure.
- A subsequent MCP `product_adr_show ADR-099` reports `status: abandoned`.
