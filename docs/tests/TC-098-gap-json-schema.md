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
---

every finding in output must have all required fields: id, code, severity, description, affected_artifacts, suggested_action. Missing fields are a test failure.