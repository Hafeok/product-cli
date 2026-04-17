---
id: TC-519
title: log graph check integration exits one on tamper
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_519_log_graph_check_integration_exits_one_on_tamper
---

## Description

`product graph check` runs log verification when `[log] verify-on-check = true` (default) and exits 1 on a tampered log.

## Setup

1. Fixture with a valid log and valid graph.
2. Out-of-band: tamper with one entry's `reason:` so its stored hash is stale.

## Steps

1. Run `product graph check` (no flags).
2. Assert exit code 1.
3. Assert the output includes the log-tamper finding with the reconciled error code.
4. Set `[log] verify-on-check = false` in `product.toml`.
5. Re-run `product graph check` and assert the log finding is no longer reported (though the log itself is still tampered).

## Invariant

Log integrity is wired into the standard graph health check by default; it is configurable but not skippable by omission.
