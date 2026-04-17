---
id: TC-509
title: log verify detects entry modification
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_509_log_verify_detects_entry_modification
last-run: 2026-04-17T15:59:54.039660448+00:00
last-run-duration: 0.3s
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
5. Assert the emitted error code is `E017` (per-entry hash mismatch, per ADR-039).

## Invariant

Any field change outside `entry-hash` is detected at the tampered entry.