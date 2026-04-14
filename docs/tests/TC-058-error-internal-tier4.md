---
id: TC-058
title: error_internal_tier4
type: scenario
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_058_error_internal_tier4"
last-run: 2026-04-14T13:40:28.280537041+00:00
---

trigger a Tier 4 path via an injected fault. Assert exit code 3 and the internal error message format.