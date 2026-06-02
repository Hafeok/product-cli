---
id: TC-885
title: product-core compiles standalone without clap or axum
type: scenario
status: passing
validates:
  features:
  - FT-107
  adrs:
  - ADR-029
phase: 6
observes:
- file
- exit-code
runner: cargo-test
runner-args: tc_885_product_core_compiles_standalone_without_clap_or_axum
last-run: 2026-06-02T19:16:30.693377638+00:00
last-run-duration: 6.9s
---

## Description

Verifies the central invariant of FT-107: `product-core` is a pure
library crate with no CLI/HTTP dependencies. A downstream consumer
(e.g. `decision-cli`) must be able to depend on it without inheriting
`clap`, `clap_complete`, `axum`, or `tower-http`.

## Procedure

1. Run `cargo metadata --no-deps --format-version 1 --manifest-path
   product-core/Cargo.toml` (read **file**: the metadata is derived
   from `product-core/Cargo.toml`; `exit-code` must be `0`).
2. Parse the returned JSON and locate the `product-core` package's
   `dependencies` array.
3. Assert the `name` field of every dependency is **not** one of
   `clap`, `clap_complete`, `axum`, `tower-http`,
   `tower_http`.
4. Run `cargo build -p product-core` and assert `exit-code` is `0`
   (proves the crate compiles on its trimmed dep set).

## Expected

- Step 1 exits `0`.
- Step 3 finds none of the forbidden crate names.
- Step 4 exits `0` and produces a `libproduct_core.rlib` artefact
  under `target/debug/deps/`.

A failure of step 3 means a forbidden dep crept into
`product-core/Cargo.toml`. A failure of step 4 means the move
left a `use clap::` or `use axum::` import inside `product-core`
that needs to be relocated to `product-cli` or `product-mcp`.