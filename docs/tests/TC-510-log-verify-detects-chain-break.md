---
id: TC-510
title: log verify detects chain break
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_510_log_verify_detects_chain_break
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

Modifying only the `prev-hash` of an entry (without changing the preceding entry) causes `product request log verify` to detect a chain break.

## Setup

1. Fixture with ≥ 2 valid log entries.
2. Out-of-band: overwrite entry N's `prev-hash` with a different valid-looking hex string and recompute/store its `entry-hash` so that per-entry verification still passes (the tamper is chain-only, not entry-only).

## Steps

1. Run `product request log verify`.
2. Assert exit code 1.
3. Assert per-entry hash check passes (entry hashes valid N/N).
4. Assert the chain check reports a break at entry N with "prev-hash in entry" and "actual hash of entry N-1" lines.
5. Assert the emitted error code is `E018` (chain break, per ADR-039).

## Invariant

Detachment from the preceding entry is detected even when each entry is individually well-hashed.