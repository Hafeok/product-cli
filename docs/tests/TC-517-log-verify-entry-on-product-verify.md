---
id: TC-517
title: log verify entry on product verify
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_517_log_verify_entry_on_product_verify
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

A successful `product verify FT-XXX` appends exactly one `verify` entry to the log.

## Setup

1. Fixture with a feature FT-042 whose TCs are configured and passing.
2. Record the current number of log entries N.

## Steps

1. Run `product verify FT-042`.
2. Assert exit code 0.
3. Read `requests.jsonl`; assert it now has N+1 entries.
4. Assert the last entry has `type: verify`, `feature: FT-042`.
5. Assert `result.tcs-run`, `result.passing`, `result.failing`, and `result.tag-created` are populated.
6. Assert `tag-created` matches the `product/FT-042/complete` tag format (ADR-036).

## Invariant

Every successful verify produces one log entry — verify is a first-class participant in the audit trail.