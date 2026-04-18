---
id: TC-582
title: adr_status_via_mcp_writes_superseded_with_bidirectional_link
type: scenario
status: passing
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
phase: 1
runner: cargo-test
runner-args: tc_582_adr_status_via_mcp_writes_superseded_with_bidirectional_link
last-run: 2026-04-18T19:07:02.858390383+00:00
last-run-duration: 0.2s
---

## Session: adr_status_via_mcp_writes_superseded_with_bidirectional_link

**Validates:** FT-046, ADR-020

### Given

A temp repository with ADR-019 in status `accepted` (sealed with a content-hash) and a new ADR-040 in status `accepted` that is about to supersede ADR-019.

### When

An MCP client calls `product_adr_status` with:

```json
{ "id": "ADR-019", "status": "superseded", "by": "ADR-040" }
```

### Then

- The call returns `{ id: "ADR-019", status: "superseded", superseded-by: ["ADR-040"], content-hash: <preserved> }`.
- The on-disk `ADR-019` front-matter shows:
  - `status: superseded`
  - `superseded-by: [ADR-040]`
  - `content-hash` unchanged (supersession does not invalidate the seal)
- The on-disk `ADR-040` front-matter shows `supersedes: [ADR-019]` (bidirectional link written in the same atomic batch per ADR-015).
- `product graph check` exits `0` — no dangling supersession edge, no cycle.
- The file-write batch is atomic: if either write fails, both ADRs are rolled back to their pre-call state.

### Cycle detection

The handler calls the same cycle-detection helper the CLI `adr_supersede` uses. If the supersession would create a cycle (e.g. ADR-040 already transitively supersedes ADR-019), the call returns `E004 supersession-cycle` and neither ADR file is modified.