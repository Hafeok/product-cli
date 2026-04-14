---
id: TC-386
title: dep_impact_transitive
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_386_dep_impact_transitive"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

FT-003 depends-on FT-001; FT-001 uses DEP-001. Assert `product impact DEP-001` includes FT-003 in transitive dependents.