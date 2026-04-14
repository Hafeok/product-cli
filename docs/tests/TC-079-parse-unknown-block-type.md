---
id: TC-079
title: parse_unknown_block_type
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
runner-args: "tc_079_parse_unknown_block_type"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

Parse a block with an unrecognised type label "X:Unknown". Assert E001 with "unrecognised block type".