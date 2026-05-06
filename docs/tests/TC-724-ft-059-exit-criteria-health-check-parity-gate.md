---
id: TC-724
title: FT-059 exit criteria — health-check parity gate
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_724_ft_059_exit_criteria
---

## Exit Criteria for FT-059

All of the following must hold before FT-059 transitions to `status: complete`.

1. **Tool registration.** `tools::build_tool_list()` returns entries with `name == "product_drift_check"` and `name == "product_preflight"`. Both have `requires_write == false`. JSON schemas match the parameter tables in the FT-059 body.
2. **Dispatch wiring.** `registry::dispatch_tool` has explicit branches for `"product_drift_check"` and `"product_preflight"` that route to functions in `read_handlers.rs`.
3. **Behavioural parity.** TC-717, TC-718, TC-720 all pass — the MCP tool produces output equivalent to the CLI's `--format json` invocation on the same repo.
4. **Error envelope.** TC-719, TC-721, TC-722 all pass — error codes E022, E023, E024 are registered in `src/error.rs` and surface through the MCP JSON-RPC error response.
5. **Documentation honesty.** TC-723 passes — every tool name advertised in the AGENTS.md "Key MCP Tools" table maps to a registered `ToolDef`.
6. **No CLI regression.** `product drift check`, `product preflight`, `product gap check`, `product graph check`, and `product impact` all produce byte-identical output to the pre-FT-059 baseline on the project's own repo.
7. **Quality gates.** `cargo t` (alias for `cargo test --no-fail-fast`), `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
8. **Runner config.** Every TC linked to FT-059 (TC-717 through TC-724) has both `runner: cargo-test` and a `runner-args` value matching the corresponding `#[test] fn` name in `tests/integration.rs`.

This TC is an `exit-criteria` checklist — when all eight items are checked and the listed TCs are passing, FT-059 is done.
