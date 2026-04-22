---
id: TC-549
title: property TC-P013 append is idempotent
type: invariant
status: passing
validates:
  features:
  - FT-041
  - FT-043
  adrs:
  - ADR-018
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_p013_append_is_idempotent
last-run: 2026-04-22T11:46:15.496146315+00:00
last-run-duration: 1.0s
---

## TC-P013 — append is idempotent (property)

For any change request whose mutations use only `op: append`, applying the request twice produces the same end state as applying it once. `append` is deduplicating by contract (ADR-038 decision 4).

⟦Σ:Types⟧{ Req≜Arbitrary⟨AppendOnlyChangeRequest⟩; State≜DocsSnapshot }
⟦Γ:Invariants⟧{
  ∀r:Req: apply(apply(r)) = apply(r)
}
⟦Λ:Scenario⟧{
  given≜fresh_repo_with_committed_docs_tree
  when≜apply(r) then apply(r) for r ∈ proptest::arb_append_only_request() ran 1000 times
  then≜snapshot(docs, after_second_apply) = snapshot(docs, after_first_apply)
}
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩