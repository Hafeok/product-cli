---
id: TC-407
title: product schema --all returns all schemas
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

Run `product schema --all`. Assert output contains all four artifact type schemas (feature, ADR, test criterion, dependency) in a single document. Assert output is valid standalone markdown.
