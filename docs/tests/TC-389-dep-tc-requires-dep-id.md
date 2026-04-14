---
id: TC-389
title: dep_tc_requires_dep_id
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-030
phase: 1
---

TC declares `requires: [DEP-005]`. Product resolves to DEP-005's availability check. Assert the resolved check command matches DEP-005 `availability-check`.