---
id: TC-527
title: log chain breaks on any deletion
type: invariant
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_527_log_chain_breaks_on_any_deletion
last-run: 2026-04-17T15:59:54.039660448+00:00
last-run-duration: 0.3s
---

## Description

Deletion of any entry from the log causes `product request log verify` to emit a chain-break finding at the entry following the deleted one.

## Formal

⟦Σ:Types⟧{
Log ≜ Entry+
Index ≜ Integer
delete ≜ ⟨Log, Index⟩ → Log
verify ≜ Log → Result
Result ≜ Ok | E018
}

⟦Γ:Invariants⟧{
∀ log ∈ Log: ∀ n ∈ Index: 0 < n < |log| ⇒ verify(delete(log, n)) = E018
}

## Property test

For all generated logs `log` of length `L ≥ 3` and all valid indices `n`:

1. Produce `log' = delete(log, n)`.
2. Run `product request log verify` on `log'`.
3. Assert exit code ≥ 1.
4. Assert the reported chain-break position corresponds to the entry that used to follow the deleted one.

Edge cases: deletion at the head (index 0) produces a break at the new head if `L ≥ 2`; deletion at the tail is the concern of TC-518 (undetectable without tag cross-reference) and is explicitly excluded from this property.

## Invariant

Mid-log deletion is always detected.