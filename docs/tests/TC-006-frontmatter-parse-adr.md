---
id: TC-006
title: frontmatter_parse_adr
type: scenario
status: passing
validates:
  features:
  - FT-002
  - FT-003
  - FT-004
  adrs:
  - ADR-002
phase: 1
runner: cargo-test
runner-args: "tc_006_frontmatter_parse_adr"
---

parse a well-formed ADR file. Assert `features`, `supersedes`, `superseded-by` deserialise correctly.