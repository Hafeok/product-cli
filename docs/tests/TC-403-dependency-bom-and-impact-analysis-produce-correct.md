---
id: TC-403
title: Dependency BOM and impact analysis produce correct output
type: exit-criteria
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 3
runner: cargo-test
runner-args: "tc_403_dependency_bom_and_impact_analysis_produce_correct_output"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

`product dep bom` produces a complete BOM with correct type groupings. `product impact DEP-001` returns affected features after feature→DEP link setup. TC `requires: [DEP-005]` resolves to the dependency's availability check without requiring a matching entry in `[verify.prerequisites]`.