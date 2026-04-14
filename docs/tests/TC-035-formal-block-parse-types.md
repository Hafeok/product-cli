---
id: TC-035
title: formal_block_parse_types
type: scenario
status: passing
validates:
  features:
  - FT-015
  adrs:
  - ADR-011
phase: 1
runner: cargo-test
runner-args: "tc_035_formal_block_parse_types"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

parse a test criterion file with a `⟦Σ:Types⟧` block. Assert all type definitions deserialise into the `TypeDef` struct with correct names and variants.