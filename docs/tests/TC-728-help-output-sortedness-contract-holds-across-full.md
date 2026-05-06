---
id: TC-728
title: help output sortedness contract holds across full surface
type: exit-criteria
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_728_help_output_sortedness_contract_holds_across_full
---

## Exit Criteria

Phase / feature gate: FT-060 ships only when every observable
surface of the CLI's help renders subcommands in alphabetical order
and a fitness test prevents regressions.

## Formal blocks

⟦Λ:ExitCriteria⟧{
  top_level_help_sorted ≜ true
  nested_groups_sorted_count = total_nested_groups_count
  fitness_test_present ≜ true
  fitness_test_blocks_regressions ≜ true
  behavioural_regressions = 0
}

⟦Ε⟧⟨δ≜0.9;φ≜85;τ≜◊⁺⟩

## Measurement

- `top_level_help_sorted` — boolean assertion driven by the
  TC-725 integration test against `product --help`.
- `nested_groups_sorted_count / total_nested_groups_count` — TC-726
  iterates every nested `Subcommand` group; the ratio must equal 1.
- `fitness_test_present` — `cli_subcommands_are_sorted` exists in
  `tests/code_quality_tests.rs` and runs as part of `cargo t`.
- `fitness_test_blocks_regressions` — manually verified once during
  feature acceptance: insert an out-of-order variant in a feature
  branch, confirm `cargo t` fails, revert.
- `behavioural_regressions` — `cargo t` reports zero new failures
  (other than help-snapshot updates intentional to this feature).

## Pass / fail

All five conditions above must hold simultaneously for the feature
to transition to `complete`.
