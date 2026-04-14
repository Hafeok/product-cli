---
id: TC-251
title: error_json_format
type: scenario
status: unimplemented
validates:
  features: 
  - FT-010
  - FT-026
  adrs:
  - ADR-013
phase: 1
---

run `product graph check --format json` on a repo with one error and one warning. Assert stderr is valid JSON matching the schema above. Assert the `errors` array has length 1 and `warnings` has length 1.