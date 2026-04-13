---
id: TC-051
title: impact_transitive
type: scenario
status: passing
validates:
  features:
  - FT-011
  - FT-016
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_051_impact_transitive"
---

FT-007 depends-on FT-001; FT-001 linked to ADR-002. Assert `product impact ADR-002` includes FT-007 in transitive dependents.