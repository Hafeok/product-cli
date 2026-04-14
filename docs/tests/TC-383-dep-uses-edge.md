---
id: TC-383
title: dep_uses_edge
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_383_dep_uses_edge"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

feature links `uses: [DEP-001]`. Assert graph contains `FT-001 →uses→ DEP-001` and reverse `DEP-001 →usedBy→ FT-001`.