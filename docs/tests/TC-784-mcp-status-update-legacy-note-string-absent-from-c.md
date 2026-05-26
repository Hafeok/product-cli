---
id: TC-784
title: mcp_status_update_legacy_note_string_absent_from_codebase
type: invariant
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_784_mcp_status_update_legacy_note_string_absent_from_codebase
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.1s
---

## Description

Grep the entire `src/` tree for the string `"Use CLI for status
updates with full side-effects"`. Assert the count is zero. This
is the structural guard that the no-op `handle_status_update` stub
has been replaced and never silently returns.

## Formal specification

⟦Σ:Types⟧{
  Path ≜ String matching ^src/.*\.rs$
  Source ≜ {p:Path | p exists in the repository tree}
  Forbidden ≜ "Use CLI for status updates with full side-effects"
}

⟦Γ:Invariants⟧{
  ∀ p ∈ Source : Forbidden ∉ read_to_string(p)
  ⇔  count(grep(Forbidden, Source)) = 0
}

⟦Ε⟧⟨δ≜1.0;φ≜∞;τ≜◊⁺⟩

## Invariant

The legacy advisory note is a structural artefact of the no-op stub.
Its presence anywhere in `src/` indicates `handle_status_update`
(or a sibling shim) is back. Failing this test must block merge.