---
id: TC-071
title: parse_types_block
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-015
  adrs:
  - ADR-016
phase: 1
---

parse `⟦Σ:Types⟧{ Node≜IRI; Role≜Leader|Follower }`. Assert two `TypeDef` entries with correct names and union type structure.