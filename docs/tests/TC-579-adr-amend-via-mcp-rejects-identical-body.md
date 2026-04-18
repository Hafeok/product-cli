---
id: TC-579
title: adr_amend_via_mcp_rejects_identical_body
type: scenario
status: unimplemented
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
  - ADR-032
phase: 1
runner: cargo-test
runner-args: tc_579_adr_amend_via_mcp_rejects_identical_body
---

## Session: adr_amend_via_mcp_rejects_identical_body

**Validates:** FT-046, ADR-032, ADR-020

### Given

A temp repository with an ADR in status `accepted`, sealed with a known `content-hash`.

### When

An MCP client calls `product_adr_amend` with:

- `id: ADR-XXX`
- `reason: "typo fix"`
- `body:` — the **exact** current body of the ADR, byte-for-byte identical.

### Then

- The call returns `E017 amendment-nothing-changed` with hint text naming the ADR and noting the content-hash already matches.
- The on-disk ADR file is byte-identical before and after the call.
- The `amendments` array in front-matter is unchanged (no empty/no-op entry appended).
- `product graph check` exits `0`.

### Rationale

The amendment audit trail must record meaningful changes only. A no-op amendment would pollute the history and allow an attacker to obscure a real amendment by surrounding it with fake ones.
