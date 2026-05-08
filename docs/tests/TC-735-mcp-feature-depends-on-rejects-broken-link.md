---
id: TC-735
title: product_feature_depends_on rejects unknown target
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_735_mcp_feature_depends_on_rejects_broken_link
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.7s
---

## Given

A repository with feature `FT-001` only.

## When

`product feature depends-on FT-001 --add FT-DOES-NOT-EXIST` runs.

## Then

- Exit code is non-zero.
- The error names the missing feature ID.
- The on-disk `FT-001` front-matter is unchanged: its `depends-on` array
  does not contain `FT-DOES-NOT-EXIST`.
- `product graph check` exits `0` after the rejected call.