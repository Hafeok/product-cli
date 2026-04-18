---
id: TC-577
title: adr_amend_via_mcp_body_and_reason_atomic
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
runner-args: tc_577_adr_amend_via_mcp_body_and_reason_atomic
---

## Session: adr_amend_via_mcp_body_and_reason_atomic

**Validates:** FT-046, ADR-032 (content-hash), ADR-020 (MCP)

### Given

A temp repository with an ADR in status `accepted` and a valid `content-hash` sealing its body.

### When

An MCP client calls `product_adr_amend` with:

```json
{
  "id": "ADR-019",
  "reason": "Remove internal LLM call per ADR-040",
  "body": "**Status:** Accepted\n\n**Context:** ... new body ...\n"
}
```

### Then

- The call returns a success response containing the new `content-hash` and the full `amendments` array including the just-recorded entry.
- The on-disk ADR file front-matter shows:
  - `content-hash: sha256:<new hash>` (different from the pre-call hash)
  - `amendments` array contains one new entry with `reason: "Remove internal LLM call per ADR-040"` and `previous-hash: sha256:<old hash>`
- The markdown body on disk is the new body supplied in the call.
- `product graph check` exits `0` after the amendment (no hash mismatch, no dangling reference).
- No intermediate state is visible on disk: either the full amendment landed or the file is unchanged (atomicity per ADR-015).
