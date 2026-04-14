---
id: TC-098
title: gap_json_schema
type: scenario
status: passing
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
runner: cargo-test
runner-args: "tc_098_gap_json_schema"
last-run: 2026-04-14T17:25:14.338071018+00:00
---

every finding in output must have all required fields: id, code, severity, description, affected_artifacts, suggested_action. Missing fields are a test failure.