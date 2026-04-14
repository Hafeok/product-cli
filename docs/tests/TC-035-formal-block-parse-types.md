---
id: TC-035
title: formal_block_parse_types
type: scenario
status: unimplemented
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
---

parse a test criterion file with a `⟦Σ:Types⟧` block. Assert all type definitions deserialise into the `TypeDef` struct with correct names and variants.