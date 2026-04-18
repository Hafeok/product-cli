---
id: TC-578
title: adr_amend_via_mcp_refuses_to_change_status
type: invariant
status: unimplemented
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
  - ADR-032
phase: 1
runner: cargo-test
runner-args: tc_578_adr_amend_via_mcp_refuses_to_change_status
---

## Invariant: adr_amend_via_mcp_refuses_to_change_status

**Validates:** FT-046, ADR-032 (content-hash governance), ADR-020 (MCP)

⟦Γ:Invariants⟧{
  adr_amend_rejects_any_payload_carrying_status_field
  adr_amend_rejects_any_payload_carrying_amendments_field
  rejected_amend_call_leaves_adr_file_unchanged
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

### Evidence

- An MCP client calls `product_adr_amend` with a payload that also supplies `status: abandoned`. The call returns `E019 amendment-carries-status`.
- An MCP client calls `product_adr_amend` with a payload whose `body` field contains inline front-matter attempting to set `status`. The call returns `E019 amendment-carries-status` (the handler parses the body as a body only — it does not merge front-matter from it).
- In both cases, a pre/post SHA-256 of the ADR file is identical.
- `product graph check` reports no changes.
- The error message names the correct alternative: `"Status transitions go through product_adr_status (accepted requires CLI)."`
