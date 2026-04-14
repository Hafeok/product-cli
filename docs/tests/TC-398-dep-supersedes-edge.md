---
id: TC-398
title: dep_supersedes_edge
type: scenario
status: unimplemented
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
---

DEP-011 supersedes DEP-005. Assert graph contains `DEP-011 →supersedes→ DEP-005`. Assert `product impact DEP-005` includes DEP-011 in dependents.