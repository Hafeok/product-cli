---
id: TC-509
title: log verify detects entry modification
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_509_log_verify_detects_entry_modification
---

## Description

Modifying any byte inside an entry (other than `entry-hash` itself) causes `product request log verify` to detect the tamper.

## Setup

1. Fixture repository with ≥ 2 valid log entries.
2. Out-of-band: rewrite entry N's `reason:` field to a different string, leaving `entry-hash` stale.

## Steps

1. Run `product request log verify`.
2. Assert exit code 1.
3. Assert stdout/stderr identifies the tampered line (line number, REQ-ID).
4. Assert the error prints the stored hash and the recomputed hash, which differ.
5. Assert the emitted error code is the reconciled per-entry-hash-mismatch code (see FT-042 code numbering note — provisionally E015, actually next free in the integrity tier).

## Invariant

Any field change outside `entry-hash` is detected at the tampered entry.
