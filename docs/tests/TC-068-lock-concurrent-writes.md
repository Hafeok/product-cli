---
id: TC-068
title: lock_concurrent_writes
type: scenario
status: unimplemented
validates:
  features:
  - FT-005
  adrs:
  - ADR-015
phase: 1
---

spawn two Product processes simultaneously, both running `product feature status FT-001 complete`. Assert exactly one succeeds and the other exits with E010. Assert the file contains a valid result from exactly one process.