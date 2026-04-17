---
id: TC-524
title: log verify is pure read
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_524_log_verify_is_pure_read
---

## Description

`product request log verify` is a pure read — even when it detects tampering, it does not modify `requests.jsonl`.

## Setup

1. Fixture with a tampered log (e.g. one modified entry, one broken chain link).
2. Snapshot the full byte content of `requests.jsonl`.

## Steps

1. Run `product request log verify`. Assert exit code 1.
2. Read `requests.jsonl` and assert its bytes are byte-identical to the snapshot.
3. Run `product request log verify --against-tags`. Assert exit code ≥ 1.
4. Re-assert the file is byte-identical to the snapshot.

## Invariant

Verification is observational. The log is never modified as a side effect of checking it, regardless of what the check finds.
