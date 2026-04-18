---
id: TC-507
title: log chain intact after multiple applies
type: scenario
status: passing
validates:
  features:
  - FT-042
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: tc_507_log_chain_intact_after_multiple_applies
last-run: 2026-04-18T10:42:11.121063959+00:00
last-run-duration: 0.2s
---

## Description

After three successive `product request apply` invocations, the chain is intact: each entry's `prev-hash` equals the preceding entry's `entry-hash`.

## Setup

1. Fixture repository with clean state.
2. Three distinct `type: create` request YAMLs.

## Steps

1. Apply request A, then B, then C in sequence.
2. Read all three lines of `requests.jsonl` in order.
3. Assert entry A's `prev-hash == "0000000000000000"`.
4. Assert entry B's `prev-hash == entry A's entry-hash`.
5. Assert entry C's `prev-hash == entry B's entry-hash`.
6. Independently recompute each entry's hash (as in TC-506) and assert they all match.

## Invariant

The chain is well-formed across arbitrary apply counts.