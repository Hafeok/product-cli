---
id: TC-539
title: session ST-020 failed-apply-leaves-zero-files
type: invariant
status: unimplemented
validates:
  features:
  - FT-041
  - FT-043
  adrs:
  - ADR-015
  - ADR-038
phase: 1
---

## ST-020 — failed apply leaves zero files

Any request with at least one E-class finding must leave every file under `docs/` byte-identical to its pre-apply state. Verified by pre/post SHA-256 checksum of every file the request could touch.

⟦Σ:Types⟧{ Req≜RequestYAML; Hash≜SHA256; File≜Path }
⟦Γ:Invariants⟧{
  ∀r:Req: findings(r) ∩ E-class ≠ ∅
    ⇒ ∀f:File: hash(f, after_apply(r)) = hash(f, before_apply(r))
}
⟦Λ:Scenario⟧{
  given≜session_with_valid_feature(FT-001)
  when≜apply(request{ type:change; target:FT-001; mutations:[{op:set; field:domains; value:[unknown-domain]}] })
  then≜apply.applied=false ∧ apply.findings contains E012 ∧ file_digests_unchanged
}
⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
