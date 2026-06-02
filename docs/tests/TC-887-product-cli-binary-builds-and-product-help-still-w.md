---
id: TC-887
title: product-cli binary builds and product --help still works
type: scenario
status: passing
validates:
  features:
  - FT-107
  adrs:
  - ADR-043
phase: 6
observes:
- stdout
- exit-code
runner: cargo-test
runner-args: tc_887_product_cli_binary_builds_and_product_help_still_works
last-run: 2026-06-02T19:16:30.693377638+00:00
last-run-duration: 0.3s
---

## Description

Verifies the user-facing surface is unchanged after the split.
`product-cli` still produces the `product` binary, and the binary
still prints the same help text on `product --help`. Any drift in
the help output is the strongest signal we accidentally changed
the CLI surface during a "pure reorganisation".

## Procedure

1. Run `cargo build -p product-cli --bin product` and assert
   `exit-code` is `0`.
2. Invoke the freshly built binary with `--help` and capture
   **stdout**.
3. Compare captured `stdout` to a checked-in fixture
   (`product-cli/tests/fixtures/product_help_v0_1_5.txt`) byte-for-byte.
   The fixture is captured on `main` (commit 4bfd6db) before the
   split lands.
4. Repeat for `product mcp --help`, `product feature --help`, and
   `product verify --help` against per-subcommand fixtures, since
   those touch the three crates respectively.

## Expected

- Build exits `0`.
- Each `--help` invocation exits `0`.
- Each captured **stdout** is byte-identical to its fixture.

A diff in step 3 or 4 means a flag, description, or default value
changed unintentionally during the move. The fix is to restore the
original definition in the relevant subcommand file, not to update
the fixture.