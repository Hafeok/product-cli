---
id: TC-512
title: log replay reconstructs state
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_512_log_replay_reconstructs_state
last-run: 2026-04-17T15:59:54.039660448+00:00
last-run-duration: 0.4s
---

## Description

`product request replay --full` reconstructs the graph from scratch into a temp directory, and the result matches the current graph on disk.

## Setup

1. Fixture repository with ≥ 5 diverse log entries (mix of create, change, verify) producing a non-trivial graph.

## Steps

1. Run `product request replay --full --output /tmp/replay-{pid}`.
2. Assert exit code 0.
3. Assert `/tmp/replay-{pid}/` exists and contains a `docs/` tree with features/, adrs/, tests/.
4. Run `product graph check --repo /tmp/replay-{pid}` and assert exit 0.
5. For every file under `docs/` in the current working tree, assert a byte-equivalent file exists at the same path in `/tmp/replay-{pid}/docs/`.

## Invariant

Replay reproduces the on-disk graph exactly. If it does not, either apply or replay has drifted — both are bugs.