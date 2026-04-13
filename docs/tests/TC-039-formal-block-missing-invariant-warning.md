---
id: TC-039
title: formal_block_missing_invariant_warning
type: invariant
status: passing
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
---

create an `invariant` type test criterion with no `⟦Γ⟧` block. Run `product graph check`. Assert exit code 2 (warning, not error).

⟦Γ:Invariants⟧{
  ∀tc:TestCriterion where tc.type = "invariant" ∧ tc.formal_blocks.is_empty():
    graph_check(tc).exit_code = 2
    ∧ graph_check(tc).warnings ⊇ {W004}
}