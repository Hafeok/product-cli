---
id: TC-780
title: mcp_feature_status_abandonment_runs_orphan_cascade
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-010
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_780_mcp_feature_status_abandonment_runs_orphan_cascade
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.1s
---

## Description

Compose a temp repo with FT-X (status `planned`) linked to TC-Y
where TC-Y's `validates.features` contains only `FT-X`. Invoke
`product_feature_status` over MCP with `id: FT-X, status:
abandoned`. Assert: (a) FT-X's status is now `abandoned`, (b)
TC-Y's `validates.features` array no longer contains `FT-X`, (c)
the MCP success envelope reports the cascade via `orphaned-tests:
[{ test_id: "TC-Y", … }]`, (d) the on-disk result is byte-identical
to running the equivalent CLI invocation on a sibling temp repo.