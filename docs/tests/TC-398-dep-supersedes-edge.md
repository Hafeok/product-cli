---
id: TC-398
title: dep_supersedes_edge
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_398_dep_supersedes_edge"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

DEP-011 supersedes DEP-005. Assert graph contains `DEP-011 →supersedes→ DEP-005`. Assert `product impact DEP-005` includes DEP-011 in dependents.