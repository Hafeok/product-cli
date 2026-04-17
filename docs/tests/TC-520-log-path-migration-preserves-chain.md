---
id: TC-520
title: log path migration preserves chain
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_520_log_path_migration_preserves_chain
last-run: 2026-04-17T15:59:54.039660448+00:00
last-run-duration: 0.2s
---

## Description

On first run of the new binary against a repo that has `.product/request-log.jsonl` but no `requests.jsonl`, Product migrates the entries forward with a valid chain and appends a migration record.

## Setup

1. Fixture with an existing `.product/request-log.jsonl` containing 3 FT-041-era entries (no `prev-hash` or `entry-hash` fields) and no `requests.jsonl`.

## Steps

1. Run any `product` command that triggers the migration (e.g. `product request log` or `product graph check`).
2. Assert `requests.jsonl` now exists at the repository root.
3. Assert it contains 4 lines: the 3 original entries (now with chained hashes computed from their canonical form) plus a 4th `migrate` entry documenting the path move.
4. Assert the chain is valid end to end (equivalent of running `product request log verify` on the new file — exit 0).
5. Assert `.product/request-log.jsonl` is either removed or preserved with a `.migrated` suffix (implementation choice — either is acceptable, but the old path must not be read again).

## Invariant

Path migration is one-shot, non-destructive to entry content, and produces a valid chain.