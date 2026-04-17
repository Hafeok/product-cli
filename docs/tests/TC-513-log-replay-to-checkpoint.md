---
id: TC-513
title: log replay to checkpoint
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_513_log_replay_to_checkpoint
last-run: 2026-04-17T15:59:54.039660448+00:00
last-run-duration: 0.3s
---

## Description

`product request replay --to REQ-ID` stops replay at the named entry (inclusive) and produces the graph state as of that entry.

## Setup

1. Fixture with three entries REQ-A, REQ-B, REQ-C in order, where REQ-A creates a feature, REQ-B adds a second feature, REQ-C abandons the first.

## Steps

1. Run `product request replay --to REQ-B --output /tmp/replay-to-b`.
2. Assert the replayed graph contains both features.
3. Assert the first feature's status is `planned` (not `abandoned`) — REQ-C was not applied.
4. Re-run `product request replay --to REQ-A --output /tmp/replay-to-a`.
5. Assert the replayed graph contains only the first feature and it is `planned`.

## Invariant

`--to` bounds replay inclusively at the named entry; nothing after that entry is applied.