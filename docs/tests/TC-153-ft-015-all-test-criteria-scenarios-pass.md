---
id: TC-153
title: FT-015 all test-criteria scenarios pass
type: exit-criteria
status: passing
validates:
  features:
  - FT-015
  adrs: []
phase: 1
runner: cargo-test
runner-args: "tc_153_ft015_exit_criteria"
last-run: 2026-04-14T14:03:36.445391644+00:00
---

## Description

All FT-015 test-criteria scenarios pass: formal block parsing (types, invariants, scenarios, evidence), error handling (unclosed delimiters, unknown block types, out-of-range values), warnings (empty blocks, missing formal blocks), raw roundtrip preservation, and context bundle integration.