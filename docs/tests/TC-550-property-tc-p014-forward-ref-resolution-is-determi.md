---
id: TC-550
title: property TC-P014 forward-ref resolution is deterministic
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
runner-args: tc_p014_forward_ref_resolution_is_deterministic
last-run: 2026-04-18T10:42:16.223089495+00:00
last-run-duration: 0.6s
---

## TC-P014 — forward-ref resolution is deterministic (property)

For any create request, resolving `ref:` values (topological sort + ID assignment) produces the same mapping of ref names to assigned IDs on every apply against an identically-shaped fresh repository.

⟦Σ:Types⟧{ Req≜Arbitrary⟨CreateRequest⟩; RefMap≜Map⟨String, String⟩ }
⟦Γ:Invariants⟧{
  ∀r:Req: resolve_refs(r, empty_repo) = resolve_refs(r, empty_repo)
  ∧ ∀r:Req: is_topological_order(assigned_ids(r))
}
⟦Λ:Scenario⟧{
  given≜two_fresh_repos_of_identical_shape
  when≜apply(r) against each for r ∈ proptest::arb_create_request() ran 1000 times
  then≜ref_map(repo_a) = ref_map(repo_b) ∧ is_topological_order(assigned_ids)
}
⟦Ε⟧⟨δ≜0.90;φ≜100;τ≜◊⁺⟩