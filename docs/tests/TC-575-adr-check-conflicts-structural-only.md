---
id: TC-575
title: adr_check_conflicts_structural_only
type: invariant
status: unimplemented
validates:
  features:
  - FT-045
  adrs:
  - ADR-022
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_575_adr_check_conflicts_structural_only
---

## Invariant: ST-132 — adr-check-conflicts-structural-only

**Validates:** FT-045, ADR-022 (amended), ADR-040 (LLM boundary invariant)

⟦Γ:Invariants⟧{
  adr_check_conflicts_makes_zero_outbound_network_calls
  adr_check_conflicts_returns_only_structural_findings
  adr_check_conflicts_completes_in_under_one_second
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

### Findings emitted by `adr check-conflicts` (structural only)

- Supersedes chain integrity (E004 if cycle detected).
- Symmetry of `superseded-by` and `supersedes` pointers.
- Domain-overlap inconsistencies with cross-cutting ADRs without an explicit acknowledgement.
- Scope-field consistency with feature link count.

### Evidence

- The `adr::check_conflicts` function contains no HTTP client and no LLM invocation (static check).
- Running `product adr check-conflicts ADR-XXX` in a sandboxed network-denied environment succeeds with the same output as in a networked environment.
