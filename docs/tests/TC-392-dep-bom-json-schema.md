---
id: TC-392
title: dep_bom_json_schema
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_392_dep_bom_json_schema"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

assert JSON BOM output contains for each dep: id, title, type, version, status, features (list), breaking-change-risk.