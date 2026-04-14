---
id: TC-036
title: formal_block_parse_invariants
type: invariant
status: passing
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
runner: cargo-test
runner-args: "tc_036_formal_block_parse_invariants"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

parse a `⟦Γ:Invariants⟧` block with a universal quantifier. Assert the parsed expression tree matches the expected structure.

⟦Γ:Invariants⟧{
  ∀b:Block where b.type = "Invariants":
    parse(b.raw).expressions.len() ≥ 1
    ∧ parse(b.raw).quantifiers ⊇ {"∀"}
}