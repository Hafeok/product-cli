---
id: TC-886
title: product-mcp depends on product-core and exposes ToolRegistry
type: scenario
status: passing
validates:
  features:
  - FT-107
  adrs:
  - ADR-020
phase: 6
observes:
- file
- exit-code
runner: cargo-test
runner-args: tc_886_product_mcp_depends_on_product_core_and_exposes_tool_registry
last-run: 2026-06-02T19:16:30.693377638+00:00
last-run-duration: 0.2s
---

## Description

Verifies that `product-mcp` is correctly layered: it depends on
`product-core` (read **file** `product-mcp/Cargo.toml`), pulls in
its own `axum` / `tower-http` / `tokio` deps, and re-exports the
public types the CLI adapter needs (`ToolRegistry` plus the
`serve_blocking` runtime helper).

## Procedure

1. Read **file** `product-mcp/Cargo.toml` and parse the
   `[dependencies]` table. Assert it contains keys `product-core`
   (with `path = "../product-core"`), `axum`, `tower-http`,
   `tokio`.
2. Assert the same file's `[dependencies]` does **not** contain
   `clap` or `clap_complete`.
3. Run `cargo build -p product-mcp` and assert `exit-code` is `0`.
4. Compile a tiny in-test consumer that does
   `use product_mcp::{ToolRegistry, serve_blocking};` and call
   `ToolRegistry::new()`. The compile is the assertion; if either
   symbol is not re-exported the test fails to build.

## Expected

- Step 1 finds all four expected deps.
- Step 2 finds neither forbidden dep.
- Step 3 exits `0`.
- Step 4 compiles cleanly; `ToolRegistry::new()` returns without
  panic.

A failure of step 1 or 4 means the MCP crate's public surface lost
a symbol the CLI adapter depends on. A failure of step 2 means a
CLI-only dep leaked into `product-mcp`.