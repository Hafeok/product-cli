---
id: TC-581
title: adr_status_via_mcp_rejects_accepted_transition
type: invariant
status: passing
validates:
  features:
  - FT-046
  adrs:
  - ADR-020
  - ADR-032
phase: 1
runner: cargo-test
runner-args: tc_581_adr_status_via_mcp_rejects_accepted_transition
last-run: 2026-04-18T19:07:02.858390383+00:00
last-run-duration: 0.1s
---

## Invariant: adr_status_via_mcp_rejects_accepted_transition

**Validates:** FT-046, ADR-020, ADR-032 (acceptance is a sealing governance step)

⟦Γ:Invariants⟧{
  mcp_product_adr_status_refuses_accepted_target_status
  mcp_error_names_the_required_cli_command
  mcp_never_silently_drops_an_accepted_status_request
  rejected_accepted_call_leaves_adr_file_unchanged
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

### Evidence

- An MCP client calls `product_adr_status` with any ADR id and `status: accepted`. The call returns `E020 status-accepted-is-manual`.
- The error message includes the exact CLI command to run: `"Accepting an ADR is a manual step. Run: product adr status ADR-XXX accepted"`.
- A pre/post SHA-256 of the ADR file is identical.
- The `content-hash` field in front-matter is unchanged.
- `product_adr_list --status proposed` still lists the ADR (it was not transitioned).

### Rationale

ADR-032 makes acceptance a sealing action: it computes the content-hash, prints impact analysis, and requires deliberate governance. Keeping this step CLI-only ensures a human is in the loop for every seal event — no automated agent can lock a decision into the content-hashed immutability regime without an explicit operator gesture.