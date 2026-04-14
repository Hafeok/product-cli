---
id: TC-046
title: context_depth_dedup
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-011
  - FT-016
  - FT-024
  - FT-014
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_046_context_depth_dedup"
last-run: 2026-04-14T13:57:28.405167723+00:00
---

two paths from FT-001 to ADR-002 (via direct link and via depends-on chain). Assert ADR-002 appears exactly once in the bundle.