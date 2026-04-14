---
id: TC-404
title: product schema returns feature front-matter schema
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_404_product_schema_returns_feature_front_matter_schema"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product schema feature`. Assert output contains all feature front-matter fields: `id`, `title`, `phase`, `status`, `depends-on`, `domains`, `adrs`, `tests`, `uses`, `domains-acknowledged`, `bundle`. Assert each field has a type description and allowed values where applicable.