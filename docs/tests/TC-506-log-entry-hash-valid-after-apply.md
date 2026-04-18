---
id: TC-506
title: log entry hash valid after apply
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_506_log_entry_hash_valid_after_apply
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

After `product request apply`, the appended entry's `entry-hash` equals `sha256(canonical_json(entry with entry-hash: ""))`.

## Setup

1. Fixture repository with clean state.
2. Apply one request.

## Steps

1. Read the last (only) line of `requests.jsonl` and parse it as a JSON object.
2. Snapshot the stored `entry-hash` value.
3. Replace `entry-hash` with `""` in the object.
4. Canonical-JSON-serialise the object (keys sorted at every level, no whitespace, UTF-8).
5. Compute `sha256` of the resulting bytes (hex, lowercase, `sha256:` prefix).
6. Assert the computed hash equals the snapshotted stored hash.

## Invariant

The stored `entry-hash` always reflects the canonical hash of the entry as written.