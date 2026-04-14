---
id: TC-071
title: parse_types_block
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-004
  - FT-015
  adrs:
  - ADR-016
phase: 1
runner: cargo-test
runner-args: "tc_071_parse_types_block"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

parse `⟦Σ:Types⟧{ Node≜IRI; Role≜Leader|Follower }`. Assert two `TypeDef` entries with correct names and union type structure.