---
id: TC-382
title: dep_parse_service
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_382_dep_parse_service"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

parse a `service` type dependency with `interface` block. Assert interface fields (protocol, port, auth, env) are present.