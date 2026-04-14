---
id: TC-407
title: product schema --all returns all schemas
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_407_product_schema_all_returns_all_schemas"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product schema --all`. Assert output contains all four artifact type schemas (feature, ADR, test criterion, dependency) in a single document. Assert output is valid standalone markdown.