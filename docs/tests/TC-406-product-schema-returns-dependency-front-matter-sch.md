---
id: TC-406
title: product schema returns dependency front-matter schema
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_406_product_schema_returns_dependency_front_matter_schema"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product schema dep`. Assert output lists all six dependency types (`library`, `service`, `api`, `tool`, `hardware`, `runtime`). Assert `interface` block is documented for service and api types. Assert `availability-check` field is described.