---
id: TC-562
title: unified_verify_pipeline_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-044
  adrs:
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_562_unified_verify_pipeline_exit
---

## Exit Criteria — FT-044 Unified Verify Pipeline

FT-044 is complete when all of the following hold:

1. `product verify` with no arguments runs all six stages and exits with 0 (pass), 1 (error), or 2 (warning) according to the worst stage result (TC-552, TC-553, TC-554).
2. Every stage runs regardless of earlier-stage failures — the report is complete (TC-553, TC-555, TC-560).
3. Stage 1 wraps `product request log verify` and reports E015/E016 as errors, W021 as warning (TC-560).
4. Stage 2 wraps `product graph check` and classifies E-class as errors, W-class as warnings (TC-553, TC-554).
5. Stage 3 checks `schema-version` in `product.toml` against the binary (E008 / W007) — covered by the schema-validation path.
6. Stage 4 wraps `product metrics threshold` and respects per-threshold `severity = error | warning` (TC-561).
7. Stage 5 runs `product verify FT-XXX` per in-scope feature; features in locked phases are skipped with a named reason (TC-555, TC-556).
8. Stage 6 wraps `product verify --platform` and reports per-TC results.
9. `product verify --phase N` scopes stage 5 to the named phase; other stages are unaffected (TC-557).
10. `product verify FT-XXX` retains exact pre-FT-044 behaviour — per-feature, writes completion tag, no pipeline report (TC-559).
11. `product verify --ci` emits a single top-level JSON document matching the documented schema with no ANSI colour (TC-558).
12. `product verify` completes in reasonable wall time on a realistic repository (no hard numeric target in this TC, but session tests keep the temp-repo fixtures small).
13. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and `cargo build` all pass.
14. Every TC in the feature (TC-552 through TC-562) has `runner: cargo-test` and `runner-args` matching the Rust test function name.
