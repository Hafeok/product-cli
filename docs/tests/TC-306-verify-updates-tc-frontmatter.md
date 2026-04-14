---
id: TC-306
title: verify_updates_tc_frontmatter
type: scenario
status: passing
validates:
  features: 
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_306_verify_updates_tc_frontmatter
last-run: 2026-04-14T18:03:59.846825760+00:00
last-run-duration: 0.2s
---

run verify. Assert `last-run`, `last-run-duration` written to TC files.