---
id: TC-008
title: frontmatter_missing_required
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
runner-args: "tc_008_frontmatter_missing_required"
---

parse a feature file with no `id` field. Assert the parser returns a structured error with the file path and field name.