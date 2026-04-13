---
id: TC-007
title: frontmatter_invalid_id
type: scenario
status: passing
validates:
  features:
  - FT-002
  - FT-003
  adrs:
  - ADR-002
phase: 1
---

parse a feature file where `adrs` references a non-existent ID. Assert `graph check` reports the broken link and exits with code 1.