---
id: TC-801
title: --dry-run prints the auto-fill plan without writing TC front-matter
type: scenario
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_801_dry_run_prints_plan_no_write
runner-timeout: 120
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.2s
---

## Scenario

A feature `FT-XXX` is `planned` and has one linked TC with no
runner config.

## When

The user runs `product implement FT-XXX --dry-run` (no
`--no-auto-runners`).

## Then

Step 0a prints the planned auto-fill to stdout as a diagnostic
line, but **no write** occurs — the TC's front-matter on disk is
unchanged. The pipeline proceeds to Step 0 (preflight), then stops
before agent invocation per the existing `--dry-run` semantics.

The combination preserves the original `--dry-run` contract: the
user can preview every planned mutation, including the auto-fill,
without committing any state. A subsequent invocation **without**
`--dry-run` will perform the writes.