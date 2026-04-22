---
id: TC-551
title: session harness and phase-1 session library pass
type: exit-criteria
status: passing
validates:
  features:
  - FT-043
  adrs:
  - ADR-018
phase: 1
runner: cargo-test
runner-args: tc_551_session_harness_and_phase_1_session_library_pass
last-run: 2026-04-22T11:46:15.496146315+00:00
last-run-duration: 0.1s
---

## Exit Criteria — Session harness and Phase 1 session library pass

The session-based testing feature is complete when:

1. The `Session` and `ApplyResult` types from `tests/sessions/mod.rs` compile and expose every method documented in `docs/product-testing-spec.md` § Session Runner.
2. `cargo test --test sessions` runs every session test under `tests/sessions/` and reports them in the test output.
3. Every session in the Phase 1 library (ST-001..ST-006, ST-020..ST-022, ST-030..ST-035) passes on a clean checkout.
4. Every property test TC-P012..TC-P014 in `tests/property.rs` passes with `PROPTEST_CASES=1000`.
5. Every TC under this feature has `runner: cargo-test` and `runner-args: "tc_XXX_snake_case"` in its front-matter, so `product verify FT-043` can execute each one.
6. `product graph check` exits 0 on the repository after all of the above.

⟦Λ:ExitCriteria⟧{
  session_harness_compiles
  cargo_test_sessions_passes_on_clean_checkout
  phase_1_session_library_complete
  property_tc_p012_p013_p014_pass_at_1000_cases
  every_tc_has_runner_config
  product_verify_FT_043_exits_zero
  product_graph_check_exits_zero
}
⟦Ε⟧⟨δ≜0.98;φ≜100;τ≜◊⁺⟩