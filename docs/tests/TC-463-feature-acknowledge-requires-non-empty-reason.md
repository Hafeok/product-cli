---
id: TC-463
title: feature acknowledge requires non-empty reason
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Run `product feature acknowledge FT-XXX --domain security` without `--reason`. Assert exit code 1 and error E011. Run with `--reason "  "` (whitespace only). Assert exit code 1 and error E011. Run with `--reason "No trust boundaries introduced"`. Assert exit code 0 and the `domains-acknowledged` block in front-matter contains the domain with the provided reasoning.