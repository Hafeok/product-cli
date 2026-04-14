---
id: TC-405
title: product schema returns ADR front-matter schema
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_405_product_schema_returns_adr_front_matter_schema"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product schema adr`. Assert output contains all ADR front-matter fields: `id`, `title`, `status`, `features`, `supersedes`, `superseded-by`, `domains`, `scope`, `source-files`. Assert status enum values are documented.