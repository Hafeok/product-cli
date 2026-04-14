---
id: TC-056
title: error_json_format
type: scenario
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_056_error_json_format"
last-run: 2026-04-14T13:40:28.280537041+00:00
---

run `product graph check --format json` on a repo with one error and one warning. Assert stderr is valid JSON matching the schema above. Assert the `errors` array has length 1 and `warnings` has length 1.