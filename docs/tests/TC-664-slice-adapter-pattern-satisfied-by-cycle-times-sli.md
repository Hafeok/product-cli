---
id: TC-664
title: slice_adapter_pattern_satisfied_by_cycle_times_slice
type: scenario
status: passing
validates:
  features:
  - FT-054
  adrs:
  - ADR-043
phase: 5
runner: cargo-test
runner-args: tc_664_slice_adapter_pattern_satisfied_by_cycle_times_slice
last-run: 2026-04-21T13:48:57.409699651+00:00
last-run-duration: 0.3s
---

## Session — slice-adapter-pattern-satisfied-by-cycle-times-slice

A concrete scenario exercising ADR-043's pinned invariants against
the `src/cycle_times/` slice + `src/commands/cycle_times.rs`
adapter introduced by FT-054. If the pattern holds for the newest
slice, the ADR's structural contract is live, not aspirational.

### Given

- The Product repository at HEAD, after FT-054 lands.
- A `src/cycle_times/` directory containing `mod.rs`,
  `compute.rs`, `render.rs`, `json.rs`, and `tests.rs`.
- A `src/commands/cycle_times.rs` adapter plus a
  `src/commands/forecast.rs` adapter.

### When

The slice + adapter fitness assertions in
`tests/code_quality_tests.rs` run against the tree.

### Then

- Every `plan_*` / `build_*` function in `src/cycle_times/` is
  a pure function: no `println!`, no `eprintln!`, no
  `std::process::exit`, no `std::fs::write`, no direct git
  process spawn (git plumbing lives in a passed-in reader).
- Every `apply_*` function in `src/cycle_times/` returns
  `Result<_, ProductError>`; any file write goes through
  `fileops::write_file_atomic` or `fileops::write_batch_atomic`.
  Cycle-times ships read-only so no `apply_*` is required; the
  invariant is satisfied vacuously.
- `src/commands/cycle_times.rs` and
  `src/commands/forecast.rs` each return `CmdResult`
  (`Result<Output, ProductError>`), not `BoxResult`.
- Each adapter file is under 400 lines (enforced by TC-402;
  asserted here for the two new adapters specifically).
- The module doc comment on every new file's first `//!` line
  contains no literal `and` token (SRP fitness check from
  `tests/code_quality_tests.rs`).

### And

Running the slice unit tests (`cargo test -p product_lib
cycle_times::tests`) exercises `plan_*` / `build_*` functions
against `KnowledgeGraph::default()` with synthetic tag
timestamps — no tempdir, no `assert_cmd`, no stdout parsing.
This is the observable payoff of ADR-043's decision: the
business logic is reachable from fast unit tests.