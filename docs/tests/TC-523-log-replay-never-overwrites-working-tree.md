---
id: TC-523
title: log replay never overwrites working tree
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_523_log_replay_never_overwrites_working_tree
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

`product request replay` never overwrites the working tree. Running replay without `--output` writes to a temp directory, and passing `--output .` or the working tree path is refused.

## Setup

1. Fixture with a valid log and a known file state in the working tree.

## Steps

1. Snapshot SHA-256 hashes of every file under `docs/`.
2. Run `product request replay --full` (no `--output`); capture the default output path from stdout.
3. Assert the default output path is outside the current working tree (e.g. under `/tmp`).
4. Re-hash every `docs/` file; assert all hashes match the snapshot (working tree untouched).
5. Run `product request replay --full --output .` and assert exit code ≥ 1 with a clear error about overwriting the working tree.
6. Re-hash every `docs/` file; assert the working tree is still untouched.

## Invariant

Replay is a read-and-reconstruct operation. It cannot destroy the working tree, even by user mistake.