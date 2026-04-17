---
id: TC-515
title: log undo does not delete entries
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_515_log_undo_does_not_delete_entries
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
