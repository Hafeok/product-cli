---
id: TC-078
title: parse_raw_roundtrip
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
runner-args: "tc_078_parse_raw_roundtrip"
---

parse an invariant block and assert that `Invariant.raw` is byte-for-byte identical to the original input (including whitespace).