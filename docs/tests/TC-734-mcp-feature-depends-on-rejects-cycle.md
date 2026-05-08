---
id: TC-734
title: product_feature_depends_on rejects cycle-creating add
type: scenario
status: passing
validates:
  features:
  - FT-062
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_734_mcp_feature_depends_on_rejects_cycle
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.6s
---

## Given

A repository with `FT-001 → FT-002 → FT-003` (chain of `depends-on`
edges) and one extra feature `FT-X`.

## When

`product feature depends-on FT-003 --add FT-001` runs (which would close
the cycle `FT-003 → FT-001 → FT-002 → FT-003`).

Self-edge variant: `product feature depends-on FT-X --add FT-X` runs.

## Then

- Both invocations exit non-zero.
- The error mentions a dependency cycle (E003 / `dependency cycle`).
- The on-disk feature files are byte-identical pre/post: `FT-001`,
  `FT-002`, `FT-003`, and `FT-X` front-matter is unchanged.
- `product graph check` exits `0` afterwards.