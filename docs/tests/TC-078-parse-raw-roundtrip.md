---
id: TC-078
title: parse_raw_roundtrip
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

parse an invariant block and assert that `Invariant.raw` is byte-for-byte identical to the original input (including whitespace).