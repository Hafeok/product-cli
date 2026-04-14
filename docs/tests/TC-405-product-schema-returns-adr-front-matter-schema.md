---
id: TC-405
title: product schema returns ADR front-matter schema
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

Run `product schema adr`. Assert output contains all ADR front-matter fields: `id`, `title`, `status`, `features`, `supersedes`, `superseded-by`, `domains`, `scope`, `source-files`. Assert status enum values are documented.
