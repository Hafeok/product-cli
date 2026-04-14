---
id: TC-381
title: dep_parse_library
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_381_dep_parse_library"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

parse a `library` type dependency. Assert all fields deserialise correctly. Assert `availability-check: ~` parses to `None`.