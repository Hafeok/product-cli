---
id: TC-050
title: impact_direct
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
runner-args: "tc_050_impact_direct"
last-run: 2026-04-14T15:03:33.506444091+00:00
---

ADR-002 linked to FT-001 and FT-004. Assert `product impact ADR-002` reports both features in direct dependents.