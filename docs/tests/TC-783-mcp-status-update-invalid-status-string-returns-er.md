---
id: TC-783
title: mcp_status_update_invalid_status_string_returns_error
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_783_mcp_status_update_invalid_status_string_returns_error
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.1s
---

## Description

Compose a temp repo with FT-X (status `planned`) and TC-Y (status
`unimplemented`). Invoke `product_feature_status` with `id: FT-X,
status: bogus`. Repeat with `product_test_status` and TC-Y.
Assert: both calls return JSON-RPC errors mentioning the invalid
status string. Neither file is modified.