---
id: TC-406
title: product schema returns dependency front-matter schema
type: scenario
status: unimplemented
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
---

## Description

Run `product schema dep`. Assert output lists all six dependency types (`library`, `service`, `api`, `tool`, `hardware`, `runtime`). Assert `interface` block is documented for service and api types. Assert `availability-check` field is described.
