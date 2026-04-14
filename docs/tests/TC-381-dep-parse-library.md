---
id: TC-381
title: dep_parse_library
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-030
phase: 1
---

parse a `library` type dependency. Assert all fields deserialise correctly. Assert `availability-check: ~` parses to `None`.