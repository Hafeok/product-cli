---
id: TC-514
title: log undo appends inverse
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_514_log_undo_appends_inverse
---

## Description

`product request undo REQ-ID` appends a new `undo` entry whose `inverse-request` reverses the target entry's mutations.

## Setup

1. Fixture with one applied entry REQ-ORIG that set `FT-009.status = in-progress`.

## Steps

1. Run `product request undo REQ-ORIG`.
2. Assert exit code 0.
3. Read the last line of `requests.jsonl` — must be a new entry with `type: undo` and `undoes: REQ-ORIG`.
4. Assert `inverse-request.changes` contains a mutation reversing the original `set` (to the prior value of `FT-009.status`).
5. Assert the on-disk FT-009 file's `status` has been restored to the prior value.
6. Assert the new entry is properly chained (prev-hash equals entry hash of REQ-ORIG, entry-hash recomputes correctly).

## Invariant

Undo is a forward operation: it appends an inverse, it does not rewrite history.
