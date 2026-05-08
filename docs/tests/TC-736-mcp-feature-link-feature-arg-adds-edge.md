---
id: TC-736
title: product_feature_link feature arg adds depends-on edge
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_736_mcp_feature_link_feature_arg_adds_edge
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.7s
---

## Given

Two features `FT-001` and `FT-002`, neither linked.

## When

`product feature link FT-001 --dep FT-002` runs (the existing one-shot
CLI form, which the MCP `product_feature_link` tool also exposes via the
new `feature` parameter — they share the same plan/apply pair).

## Then

- Exit code is `0`.
- `FT-001`'s `depends-on` contains `FT-002`.
- Re-running the same command is idempotent — no duplicate entry, exit
  code `0`.
- `product graph check` exits `0`.