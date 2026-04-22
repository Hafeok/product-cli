---
id: TC-541
title: session ST-022 concurrent-apply-serialised by advisory lock
type: chaos
status: passing
validates:
  features:
  - FT-041
  - FT-043
  adrs:
  - ADR-015
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_541_session_st_022_concurrent_apply_serialised_by_advisory_lock
last-run: 2026-04-22T11:46:15.496146315+00:00
last-run-duration: 3.1s
---

## ST-022 — concurrent apply serialised by advisory lock

Two simultaneous `product request apply` invocations against the same repository must be serialised by the advisory lock from ADR-015. Exactly one apply succeeds per round-trip; the other either waits and then succeeds, or fails with E010 if the lock-wait budget is exceeded.

⟦Σ:Types⟧{ Proc≜ApplyInvocation; ExitCode≜Int }
⟦Γ:Invariants⟧{
  ∀p1,p2:Proc overlapping_in_time: exit_code(p1)=0 ⊕ exit_code(p2)=0 ⊕ (exit_code(p1)=0 ∧ exit_code(p2) ∈ {0, E010})
}
⟦Λ:Scenario⟧{
  given≜session_with_clean_graph
  when≜spawn_two_apply_processes_with_same_request(request)
  then≜successful_count ≥ 1 ∧ graph_valid_after
}
⟦Ε⟧⟨δ≜0.85;φ≜100;τ≜◊⁺⟩