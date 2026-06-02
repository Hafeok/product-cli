---
id: TC-889
title: external crate can depend on product-core alone
type: scenario
status: passing
validates:
  features:
  - FT-107
  adrs:
  - ADR-029
phase: 6
observes:
- exit-code
- file
runner: cargo-test
runner-args: tc_889_external_crate_can_depend_on_product_core_alone
last-run: 2026-06-02T19:16:30.693377638+00:00
last-run-duration: 0.3s
---

## Description

The motivating use case for FT-107: a sibling CLI (e.g.
`decision-cli`) wants the graph engine and slice library without
inheriting any of the CLI or MCP surface. This TC pins the
contract by shipping a tiny fixture consumer alongside the
workspace.

## Procedure

1. The fixture crate lives at
   `product-cli/tests/fixtures/external-core-consumer/` (read
   **file**: it is a real `Cargo.toml` + `src/main.rs` checked
   into the repo). Its `Cargo.toml` declares exactly one
   dependency: `product-core = { path = "../../../../product-core" }`.
2. Run `cargo build --manifest-path
   product-cli/tests/fixtures/external-core-consumer/Cargo.toml`
   and assert **exit-code** is `0`.
3. Run the resulting binary and assert **exit-code** is `0`. The
   binary loads `KnowledgeGraph::load_from_root(temp_dir)?` against
   a freshly-initialised tempdir, then calls
   `fileops::write_file_atomic` to write a smoke file. Both calls
   are public API exposed by `product-core`.
4. Read **file** at the path the consumer wrote and assert it
   contains the expected sentinel string (`"external-core ok"`).
5. Run `cargo metadata --no-deps --format-version 1
   --manifest-path
   product-cli/tests/fixtures/external-core-consumer/Cargo.toml`
   and assert the resolved dependency tree contains
   `product-core` but **does not** contain
   `product-mcp`, `product-cli`, `clap`, `axum`, or `tower-http`.

## Expected

- Steps 2 and 3 exit `0`.
- Step 4 finds the sentinel on disk (the consumer's write went
  through — causation, not just response).
- Step 5 confirms the dependency cone of `product-core` is
  CLI/HTTP-free.

This TC asserts on the actual file the consumer wrote (per
PAT-003 — read the file the action claims to mutate), not just
the consumer's exit code. A green exit code with a missing
sentinel would mean the public API silently failed.