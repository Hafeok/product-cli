---
id: TC-602
title: tc_type_invariant_requires_formal_block
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
---

## Session: ST-181 — tc-type-invariant-requires-formal-block

### Given
A TC with `type: invariant` whose body contains no `⟦Γ:Invariants⟧` and no
`⟦Σ:Types⟧` block.

### When
`product graph check` runs.

### Then
- W004 is emitted naming the TC.
- Adding a `⟦Γ:Invariants⟧` block to the TC body and re-running clears
  W004.
- Same TC with `type: scenario` does NOT trigger W004 (mechanic is
  type-specific).
