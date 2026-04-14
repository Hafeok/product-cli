---
id: TC-404
title: product schema returns feature front-matter schema
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

Run `product schema feature`. Assert output contains all feature front-matter fields: `id`, `title`, `phase`, `status`, `depends-on`, `domains`, `adrs`, `tests`, `uses`, `domains-acknowledged`, `bundle`. Assert each field has a type description and allowed values where applicable.
