---
id: TC-525
title: log entry hash is deterministic
type: invariant
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_525_log_entry_hash_is_deterministic
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

Entry hash is deterministic: serialising and hashing the same entry twice produces byte-identical output.

## Formal

⟦Σ:Types⟧{
Entry ≜ ⟨id: String, applied-at: String, type: EntryType, prev-hash: String, entry-hash: String, payload: Json⟩
Bytes ≜ String
Hash ≜ String
canonical_json ≜ Entry → Bytes
hash ≜ Entry → Hash
}

⟦Γ:Invariants⟧{
∀ e ∈ Entry: canonical_json(e) = canonical_json(e)
∀ e ∈ Entry: hash(e) = hash(e)
}

## Property test

For all generated entry values `e` (via `proptest` strategy over the entry schema — random type, random fields, random nested values, random key orderings on input maps):

1. Compute `c1 = canonical_json(e with entry-hash="")`.
2. Compute `c2 = canonical_json(e with entry-hash="")` (second call).
3. Assert `c1 == c2` (byte equality).
4. Compute `h1 = sha256(c1)` and `h2 = sha256(c2)`.
5. Assert `h1 == h2`.

## Pinning test

In addition to the property, a fixed-fixture test asserts that a specific hand-constructed entry serialises to a specific byte sequence — this catches silent behaviour changes in the JSON library across dependency upgrades.

## Invariant

Canonical JSON is a function, not a procedure. Same input, same output, forever.