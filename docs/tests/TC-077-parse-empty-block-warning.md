---
id: TC-077
title: parse_empty_block_warning
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
runner-args: "tc_077_parse_empty_block_warning"
---

Parse an invariants block with an empty body. Assert W004. Assert no error.