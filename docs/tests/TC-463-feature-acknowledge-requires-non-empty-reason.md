---
id: TC-463
title: feature acknowledge requires non-empty reason
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_463_feature_acknowledge_requires_nonempty_reason"
last-run: 2026-04-15T11:45:30.776758337+00:00
last-run-duration: 0.2s
---

Run `product feature acknowledge FT-XXX --domain security` without `--reason`. Assert exit code 1 and error E011. Run with `--reason "  "` (whitespace only). Assert exit code 1 and error E011. Run with `--reason "No trust boundaries introduced"`. Assert exit code 0 and the `domains-acknowledged` block in front-matter contains the domain with the provided reasoning.