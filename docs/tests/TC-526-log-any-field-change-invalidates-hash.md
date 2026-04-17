---
id: TC-526
title: log any field change invalidates hash
type: invariant
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_526_log_any_field_change_invalidates_hash
last-run: 2026-04-17T15:59:54.039660448+00:00
last-run-duration: 0.3s
---

## Description

Any mutation of any field in an entry (except `entry-hash` itself) invalidates the stored hash.

## Formal

⟦Σ:Types⟧{
Entry ≜ ⟨id: String, applied-at: String, type: EntryType, prev-hash: String, entry-hash: String, payload: Json⟩
Field ≜ String
Value ≜ Json
mutate ≜ ⟨Entry, Field, Value⟩ → Entry
hash ≜ Entry → String
}

⟦Γ:Invariants⟧{
∀ e ∈ Entry: ∀ f ∈ Field: ∀ v ∈ Value: f ≠ entry-hash ∧ v ≠ e.f ⇒ hash(mutate(e,f,v)) ≠ e.entry-hash
}

## Property test

For all generated triples `(e, f, v)` where `f` is a randomly-selected field path in `e` other than `entry-hash` and `v` is a randomly-selected value differing from `e[f]`:

1. Start with a valid entry `e` whose `entry-hash` is correct.
2. Construct `e' = mutate(e, f, v)`.
3. Compute `h' = sha256(canonical_json(e' with entry-hash=""))`.
4. Assert `h' ≠ e.entry-hash`.

This is the tamper-detection guarantee at the formal level.

## Invariant

No silent mutation: every field affects the hash.