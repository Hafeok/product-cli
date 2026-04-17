---
id: TC-548
title: property TC-P012 failed apply leaves zero files changed
type: invariant
status: unimplemented
validates:
  features:
  - FT-041
  - FT-043
  adrs:
  - ADR-015
  - ADR-018
  - ADR-038
phase: 1
---

## TC-P012 — failed apply leaves zero files changed (property)

For any randomly-generated request whose validation returns at least one E-class finding, the set of files under `docs/` is bitwise-identical before and after the apply call.

⟦Σ:Types⟧{ Req≜Arbitrary⟨RequestYAML⟩; FileSet≜Set⟨(Path, Hash)⟩ }
⟦Γ:Invariants⟧{
  ∀r:Req: ∃f∈findings(r): severity(f)=E
    ⇒ snapshot(docs, after(apply(r))) = snapshot(docs, before(apply(r)))
}
⟦Λ:Scenario⟧{
  given≜fresh_repo_with_committed_docs_tree
  when≜apply(proptest::arb_invalid_request()) ran 1000 times
  then≜∀run: snapshot_after = snapshot_before
}
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
