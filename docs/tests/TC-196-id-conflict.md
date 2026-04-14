---
id: TC-196
title: id_conflict
type: scenario
status: unimplemented
validates:
  features: 
  - FT-001
  - FT-004
  - FT-009
  adrs:
  - ADR-005
phase: 1
---

attempt to create a feature with an ID that already exists. Assert the CLI returns an error and does not overwrite the existing file.