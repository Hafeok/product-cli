---
id: TC-805
title: FT-068 consolidated build test lint exit criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_805_ft_068_consolidated_exit_criteria
runner-timeout: 600
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.2s
---

## Exit criteria

Consolidated build, test, and lint gates after FT-068 lands.

All of the following must pass:

- `cargo build` succeeds.
- `cargo t` (the `--no-fail-fast` alias in `.cargo/config.toml`)
  runs every test binary — unit, doc, code_quality, integration,
  property, sessions — and returns success.
- `cargo clippy -- -D warnings -D clippy::unwrap_used` returns
  success.
- `tests/code_quality_tests.rs` passes: no source file exceeds
  400 lines, no module doc comment's first `//!` line contains the
  word "and".
- `product graph check` returns clean (no E-class findings) on
  the post-FT-068 graph.
- `product verify FT-068` reports every linked TC (TC-799 through
  TC-805) passing.

This TC is the rollup gate the harness checks after the agent
completes the implementation. It carries no behaviour of its own —
it is the per-feature mirror of the project-wide three-gates rule
in CLAUDE.md.