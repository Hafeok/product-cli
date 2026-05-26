---
id: TC-781
title: mcp_feature_status_in_progress_blocked_by_tc_runner_gate
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_781_mcp_feature_status_in_progress_blocked_by_tc_runner_gate
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.1s
---

## Description

Compose a temp repo with FT-X (status `planned`) linked to TC-Y
that lacks `runner` and `runner-args`. Invoke
`product_feature_status` over MCP with `id: FT-X, status:
in-progress`. Assert: (a) the MCP response is a JSON-RPC error
whose message contains `TcRunnerMissing` and names `TC-Y`, (b)
FT-X's status on disk is still `planned`, (c) TC-Y is unmodified.