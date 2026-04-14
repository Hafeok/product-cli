---
id: TC-293
title: gap_json_schema
type: scenario
status: unimplemented
validates:
  features: 
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

every finding in output must have all required fields: id, code, severity, description, affected_artifacts, suggested_action. Missing fields are a test failure.