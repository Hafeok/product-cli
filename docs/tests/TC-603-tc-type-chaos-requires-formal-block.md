---
id: TC-603
title: tc_type_chaos_requires_formal_block
type: scenario
status: unimplemented
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
---

## Session: ST-182 — tc-type-chaos-requires-formal-block

### Given
A TC with `type: chaos` whose body contains no formal block.

### When
`product graph check` runs.

### Then
- W004 is emitted naming the TC.
- Adding a `⟦Γ:Invariants⟧` or `⟦Σ:Types⟧` block clears W004.
