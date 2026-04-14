---
id: TC-072
title: parse_invariants_block
type: invariant
status: passing
validates:
  features:
  - FT-003
  - FT-004
  - FT-015
  adrs:
  - ADR-016
phase: 1
runner: cargo-test
runner-args: "tc_072_parse_invariants_block"
last-run: 2026-04-14T10:46:07.489682314+00:00
---

parse a block with a universal quantifier. Assert `Invariant.raw` matches the input verbatim.

⟦Γ:Invariants⟧{
  ∀b:Block where b.type = "Invariants":
    roundtrip(b) = b.raw
    ∧ parse(b.raw).is_ok()
}