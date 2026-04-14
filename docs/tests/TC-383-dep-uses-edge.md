---
id: TC-383
title: dep_uses_edge
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-030
phase: 1
---

feature links `uses: [DEP-001]`. Assert graph contains `FT-001 →uses→ DEP-001` and reverse `DEP-001 →usedBy→ FT-001`.