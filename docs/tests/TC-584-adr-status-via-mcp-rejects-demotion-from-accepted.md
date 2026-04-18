---
id: TC-584
title: adr_status_via_mcp_rejects_demotion_from_accepted
type: scenario
status: passing
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
  - ADR-032
phase: 1
runner: cargo-test
runner-args: tc_584_adr_status_via_mcp_rejects_demotion_from_accepted
last-run: 2026-04-18T19:07:02.858390383+00:00
last-run-duration: 0.2s
---

## Session: adr_status_via_mcp_rejects_demotion_from_accepted

**Validates:** FT-046, ADR-020, ADR-032 (immutability of accepted ADRs)

### Given

A temp repository with ADR-077 in status `accepted`, sealed with a content-hash, with a populated `amendments` array.

### When

An MCP client calls `product_adr_status` with `id: ADR-077, status: proposed`.

### Then

- The call returns `E021 status-cannot-demote-accepted` with hint text: `"Accepted ADRs cannot return to 'proposed'. Use supersede or abandon."`.
- The on-disk ADR-077 front-matter is byte-identical before and after the call — `status: accepted`, `content-hash` preserved, `amendments` preserved.
- `product graph check` exits `0` (no change).

### Rationale

ADR-032 locks accepted ADRs into an immutable regime. Demotion to `proposed` would effectively unseal the ADR and allow silent body rewrites. The only paths out of `accepted` are:
- **Amendment** — `product_adr_amend` with `body` + `reason` (logged, hashed).
- **Supersession** — `product_adr_status status=superseded by=ADR-YYY`.
- **Abandonment** — `product_adr_status status=abandoned`.

No path takes an accepted ADR back to proposed. This TC is the MCP-side guard.