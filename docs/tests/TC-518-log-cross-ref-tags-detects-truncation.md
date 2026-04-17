---
id: TC-518
title: log cross ref tags detects truncation
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_518_log_cross_ref_tags_detects_truncation
---

## Description

`product request log verify --against-tags` detects tail truncation by finding git tags with no corresponding log entry.

## Setup

1. Fixture with a clean log containing a `verify` entry for FT-009 and a corresponding `product/FT-009/complete` git tag.
2. Out-of-band: truncate the last line of `requests.jsonl` (removing the verify entry).

## Steps

1. Run `product request log verify --against-tags`.
2. Assert exit code 1 (or warning exit, per ADR-009 convention — provisionally W021 is a warning so exit 2).
3. Assert the output reports `product/FT-009/complete` as having no corresponding log entry.
4. Assert the reconciled tag-without-entry warning code is emitted (provisionally W021).

## Invariant

Truncation from the tail is invisible to chain verification alone; the git-tag cross-reference closes that gap.
