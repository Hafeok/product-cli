---
id: TC-511
title: log verify detects entry deletion
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_511_log_verify_detects_entry_deletion
---

## Description

Deleting any entry from `requests.jsonl` causes the following entry's `prev-hash` to no longer match, and `product request log verify` detects the chain break.

## Setup

1. Fixture with ≥ 3 valid log entries.
2. Out-of-band: remove line 2 from `requests.jsonl`.

## Steps

1. Run `product request log verify`.
2. Assert exit code 1.
3. Assert the chain-break error points at the line following the deletion.
4. Assert the preserved entries still individually hash correctly.
5. Assert the emitted error code is the reconciled chain-break code (provisionally E016).

## Invariant

Deletion is indistinguishable from modification-of-prev-hash for the chain check — both surface as E016-equivalents.
