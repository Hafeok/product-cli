---
id: TC-505
title: log entry appended on apply
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_505_log_entry_appended_on_apply
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

`product request apply` on a valid request YAML appends exactly one new line to `requests.jsonl`.

## Setup

1. Fixture repository with zero existing log entries (no `requests.jsonl` yet).
2. A minimal `type: create` request YAML with `reason: "test"` creating one feature.

## Steps

1. Run `product request apply request.yaml`.
2. Check `requests.jsonl` exists at the repository root.
3. Count lines — must equal 1.
4. Parse the single line as JSON; assert `type == "create"`, `reason == "test"`, and both `prev-hash` and `entry-hash` are present.
5. Assert `prev-hash == "0000000000000000"` (genesis).

## Invariant

Exactly one new line per successful apply, regardless of how many artifacts the request creates or changes.