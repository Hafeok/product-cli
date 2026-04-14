---
id: TC-226
title: formal_block_missing_invariant_warning
type: invariant
status: unimplemented
validates:
  features: 
  - FT-015
  adrs:
  - ADR-011
phase: 1
---

create an `invariant` type test criterion with no `⟦Γ⟧` block. Run `product graph check`. Assert exit code 2 (warning, not error).