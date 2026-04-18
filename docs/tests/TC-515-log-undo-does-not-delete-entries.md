---
id: TC-515
title: log undo does not delete entries
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_515_log_undo_does_not_delete_entries
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

After `product request undo REQ-ORIG`, the original entry REQ-ORIG is still present in `requests.jsonl` — undo never deletes.

## Setup

1. Fixture with one applied entry REQ-ORIG.
2. Snapshot the full byte content of REQ-ORIG's line.

## Steps

1. Run `product request undo REQ-ORIG`.
2. Read `requests.jsonl`; assert exactly 2 lines.
3. Assert line 1 is byte-identical to the snapshotted REQ-ORIG content.
4. Assert line 2 is a new `undo` entry.

## Invariant

`requests.jsonl` is append-only: undo appends a reversal, it never mutates existing entries.