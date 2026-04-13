---
id: TC-005
title: frontmatter_parse_feature
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
runner-args: "tc_005_frontmatter_parse_feature"
---

parse a well-formed feature file. Assert all fields deserialise correctly into the `Feature` struct. Assert `adrs` and `tests` vectors contain the expected IDs.