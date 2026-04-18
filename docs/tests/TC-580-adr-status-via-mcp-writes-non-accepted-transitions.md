---
id: TC-580
title: adr_status_via_mcp_writes_non_accepted_transitions
type: scenario
status: passing
validates:
  features:
  - FT-046
  adrs:
  - ADR-015
  - ADR-020
phase: 1
runner: cargo-test
runner-args: tc_580_adr_status_via_mcp_writes_non_accepted_transitions
last-run: 2026-04-18T19:07:02.858390383+00:00
last-run-duration: 0.2s
---

## Session: adr_status_via_mcp_writes_non_accepted_transitions

**Validates:** FT-046, ADR-020, ADR-015 (atomic writes)

### Given

A temp repository with an ADR-042 in status `proposed`.

### When

An MCP client calls `product_adr_status` with `id: ADR-042, status: abandoned`.

### Then

- The call returns `{ id: "ADR-042", status: "abandoned", ... }` with no `note` field recommending the CLI.
- The on-disk ADR file front-matter shows `status: abandoned`.
- No `content-hash` was computed (abandoning is not a sealing action).
- A second call to `product_adr_show ADR-042` reports `status: abandoned`.
- `product graph check` exits `0`.

### Variants to cover

The same test exercises every non-`accepted` transition to prove parity:

- `proposed → abandoned` ✓ (primary flow above)
- `accepted → abandoned` (covered by TC-583)
- `proposed → superseded` and `accepted → superseded` (covered by TC-582, includes bidirectional link)