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
runner: cargo-test
runner-args: "tc_039_formal_block_missing_invariant_warning"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

create an `invariant` type test criterion with no formal invariants block. Run `product graph check`. Assert exit code 2 (warning, not error).

⟦Γ:Invariants⟧{
  ∀tc:TestCriterion where tc.type = "invariant" ∧ tc.formal_blocks.is_empty():
    graph_check(tc).exit_code = 2
    ∧ graph_check(tc).warnings ⊇ {W004}
}