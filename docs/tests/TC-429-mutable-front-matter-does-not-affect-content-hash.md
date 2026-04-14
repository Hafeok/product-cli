---
id: TC-429
title: Mutable front-matter does not affect content-hash
type: invariant
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_429_mutable_front_matter_does_not_affect_content_hash"
---

## Description

For any accepted ADR with a valid content-hash, modifying only mutable front-matter fields must not cause a hash mismatch.

## Formal Specification

⟦Γ:Invariants⟧{
  ∀a:ADR where a.status = "accepted" ∧ a.content-hash ≠ ∅:
    ∀f ∈ {status, superseded-by, features, domains, scope, source-files}:
      let H₀ = compute_hash(a)
      ∧ modify(a.f, v') → compute_hash(a) = H₀

  ∀t:TC where t.content-hash ≠ ∅:
    ∀g ∈ {status, last-run, failure-message, last-run-duration, validates.features, runner, runner-args, runner-timeout, requires, phase}:
      let H₁ = compute_hash(t)
      ∧ modify(t.g, v') → compute_hash(t) = H₁
}
