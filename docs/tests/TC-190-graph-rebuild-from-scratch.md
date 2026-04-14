---
id: TC-190
title: graph_rebuild_from_scratch
type: scenario
status: unimplemented
validates:
  features: 
  - FT-016
  adrs:
  - ADR-003
phase: 1
---

start with a directory of 10 feature files, 8 ADR files, and 15 test files. Invoke any CLI command. Assert the graph contains the correct node and edge counts without any prior `graph rebuild` having been run.