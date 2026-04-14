---
id: TC-397
title: dep_check_manual
type: scenario
status: unimplemented
validates:
  features: []
  adrs:
  - ADR-030
phase: 1
---

run `product dep check DEP-005` with availability check that exits 0. Assert output shows check passed. With exit 1: assert shows failed.