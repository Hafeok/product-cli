---
id: TC-036
title: formal_block_parse_invariants
type: invariant
status: unimplemented
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
---

parse a `⟦Γ:Invariants⟧` block with a universal quantifier. Assert the parsed expression tree matches the expected structure.

⟦Γ:Invariants⟧{
  ∀b:Block where b.type = "Invariants":
    parse(b.raw).expressions.len() ≥ 1
    ∧ parse(b.raw).quantifiers ⊇ {"∀"}
}