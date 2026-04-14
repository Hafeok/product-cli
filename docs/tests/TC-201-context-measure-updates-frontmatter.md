---
id: TC-201
title: context_measure_updates_frontmatter
type: scenario
status: passing
runner: cargo-test
runner-args: "tc_201_context_measure_updates_frontmatter"
validates:
  features: 
  - FT-011
  adrs:
  - ADR-006
phase: 1
last-run: 2026-04-14T13:57:28.405167723+00:00
---

run `product context FT-001 --measure`. Assert feature front-matter `bundle` block is written with correct `depth-1-adrs`, `tcs`, `domains`, `tokens-approx`, and `measured-at` fields.