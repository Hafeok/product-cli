---
id: TC-181
title: ft_026_ci_integration_pass
type: exit-criteria
status: passing
validates:
  features:
  - FT-014
  - FT-026
  adrs:
  - ADR-009
  - ADR-013
phase: 3
runner: cargo-test
runner-args: "tc_181_ft_026_ci_integration_pass"
last-run: 2026-04-14T15:02:16.595537282+00:00
---

Run `product graph check --format json` and `product feature list --format json`. Both produce valid JSON output to stdout. The graph check CI gate fails on a PR with a broken link.