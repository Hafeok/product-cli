---
id: TC-782
title: mcp_status_update_unknown_id_returns_error
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_782_mcp_status_update_unknown_id_returns_error
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.1s
---

## Description

Compose a minimal temp repo. Invoke `product_feature_status` with
`id: FT-999, status: complete` (where FT-999 does not exist).
Repeat with `product_test_status` and `id: TC-999`. Assert: both
calls return a JSON-RPC error whose message contains `not found`
(case-insensitive) and the corresponding artifact ID. Neither call
returns a success envelope.