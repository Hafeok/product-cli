---
id: TC-811
title: ft_069_exit_criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-069
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_811_ft_069_exit_criteria
last-run: 2026-05-27T11:04:45.120555493+00:00
last-run-duration: 0.2s
---

## Exit Criteria

FT-069 is complete when **all** of the following hold:

1. The shared library function `graph::full_check::run` exists and
   is the single entry point for `product_graph_check` (MCP) and
   `product graph check` (CLI).
2. `src/commands/graph_cmd.rs::graph_check` delegates exclusively
   to `graph::full_check::run` and no longer contains
   `append_log_findings_to_check`.
3. `src/mcp/registry.rs` dispatches `product_graph_check` through
   `graph::full_check::run` — no inline `graph.check()` call
   remains.
4. TC-806, TC-807, TC-808, TC-809, and TC-810 all pass.
5. `cargo build` succeeds.
6. `cargo t` (the `test --no-fail-fast` alias) reports green across
   all six binaries.
7. `cargo clippy -- -D warnings -D clippy::unwrap_used` passes
   with zero warnings.
8. `product graph check` exits 0 on a clean fixture and exits 2 on
   a fixture containing only warnings (CLI exit-code semantics
   unchanged).
9. Every TC in this feature carries `runner: cargo-test` and
   `runner-args: tc_XXX_snake_case_title` matching the integration
   test function name.

## Formal

⟦Λ:ExitCriteria⟧{
  passing(TC-806) ∧ passing(TC-807) ∧ passing(TC-808)
    ∧ passing(TC-809) ∧ passing(TC-810)
  cargo_build_status = success
  cargo_test_status  = green
  clippy_warnings    = 0
  clippy_unwrap_used = 0
  cli_exit_code(clean_fixture)            = 0
  cli_exit_code(warning_only_fixture)     = 2
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩