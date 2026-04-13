---
id: TC-072
title: parse_invariants_block
type: invariant
status: passing
validates:
  features:
  - FT-003
  - FT-015
  adrs:
  - ADR-016
phase: 1
---

parse a block with a universal quantifier. Assert `Invariant.raw` matches the input verbatim.