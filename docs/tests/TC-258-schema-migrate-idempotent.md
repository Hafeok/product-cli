---
id: TC-258
title: schema_migrate_idempotent
type: scenario
status: unimplemented
validates:
  features: 
  - FT-003
  - FT-008
  - FT-020
  adrs:
  - ADR-014
phase: 1
---

run `product migrate schema` twice. Assert the second run reports zero files changed.