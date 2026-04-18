---
id: TC-508
title: log verify passes on clean log
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_508_log_verify_passes_on_clean_log
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

`product request log verify` on an untampered log exits 0 with per-entry and chain-integrity counts.

## Setup

1. Fixture repository with N ≥ 3 valid log entries produced by successive applies.

## Steps

1. Run `product request log verify`.
2. Assert exit code 0.
3. Assert stdout contains `Entry hashes valid (N/N)` with the correct N.
4. Assert stdout contains `Hash chain intact (N/N)`.
5. Assert stdout contains a final `Log is tamper-free.` line.

## Invariant

Clean logs always verify cleanly.