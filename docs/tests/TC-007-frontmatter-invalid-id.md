---
id: TC-007
title: frontmatter_invalid_id
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
runner-args: "tc_007_frontmatter_invalid_id"
last-run: 2026-04-14T10:46:07.489682314+00:00
---

parse a feature file where `adrs` references a non-existent ID. Assert `graph check` reports the broken link and exits with code 1.