---
id: TC-516
title: log migrate entry first
type: migration
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_516_log_migrate_entry_first
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

When `product migrate` runs from monolithic docs, the first log entry produced is a `migrate` entry with `prev-hash: "0000000000000000"`.

## Setup

1. Fixture with a monolithic `product-prd.md` + `product-adrs.md` and no existing `requests.jsonl`.

## Steps

1. Run `product migrate`.
2. Read `requests.jsonl`; assert the first line is an entry with `type: migrate`.
3. Assert its `prev-hash` is `"0000000000000000"` (genesis sentinel).
4. Assert `sources` lists the input files.
5. Assert `result.created` lists the migrated artifacts.

## Invariant

Migration is always the first (genesis) entry. A repo's log begins with a documented origin.