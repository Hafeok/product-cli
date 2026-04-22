---
id: TC-540
title: session ST-021 failed-apply-mid-write-recovery
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
runner-args: tc_540_session_st_021_failed_apply_mid_write_recovery
last-run: 2026-04-22T11:30:45.579976087+00:00
last-run-duration: 0.2s
---

## ST-021 — failed apply mid-write recovery

If the apply pipeline is interrupted between step 6 (sidecar writes) and step 9 (batch rename), the working tree must remain equivalent to the pre-apply state. Interruption is simulated by injecting a failure in one sidecar write; the harness then verifies no target file was renamed.

⟦Σ:Types⟧{ Step≜{6,7,8,9}; Outcome≜original|complete }
⟦Γ:Invariants⟧{
  ∀r:Req, s:Step: interrupt_at(apply(r), s)
    ⇒ ∀f:File(r): state(f) ∈ {original} ∧ ¬exists(sidecar(f))
}
⟦Λ:Scenario⟧{
  given≜session_with_valid_feature(FT-001)
  when≜inject_write_failure_at_step(6) ∧ apply(request{ type:change; target:FT-001; mutations:[{op:set; field:body; value:new_body}] })
  then≜apply.applied=false ∧ file_digest(FT-001) = pre_apply_digest ∧ no_sidecar_files_remain
}
⟦Ε⟧⟨δ≜0.90;φ≜100;τ≜◊⁺⟩