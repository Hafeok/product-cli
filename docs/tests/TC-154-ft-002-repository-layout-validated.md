---
id: TC-154
title: FT-002 repository layout validated
type: exit-criteria
status: passing
validates:
  features:
  - FT-002
  adrs:
  - ADR-002
  - ADR-004
phase: 1
runner: cargo-test
runner-args: "tc_154_ft002_exit_criteria"
last-run: 2026-04-14T13:16:43.783509783+00:00
---

## Description

All FT-002 repository layout scenarios pass: front-matter parsing for features (TC-005) and ADRs (TC-006), invalid ID detection (TC-007), missing required field detection (TC-008), markdown front-matter stripping (TC-011), and markdown passthrough (TC-012). The exit criteria validates that the repository layout is correctly initialized, queryable via `feature list` and `feature show`, and that all graph relationships are clean via `graph check`.