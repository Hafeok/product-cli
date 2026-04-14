---
id: TC-385
title: dep_impact_direct
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_385_dep_impact_direct"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

DEP-001 linked to FT-001 and FT-002. Assert `product impact DEP-001` names both features as direct dependents.