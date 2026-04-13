---
id: TC-181
title: ft_026_ci_integration_pass
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-026
  adrs:
  - ADR-009
  - ADR-013
phase: 3
---

Run `product graph check --format json` and `product feature list --format json`. Both produce valid JSON output to stdout. The graph check CI gate fails on a PR with a broken link.
