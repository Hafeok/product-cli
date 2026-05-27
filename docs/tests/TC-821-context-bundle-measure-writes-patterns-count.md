---
id: TC-821
title: context_bundle_measure_writes_patterns_count
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_821_context_bundle_measure_writes_patterns_count
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo with FT-100 citing two patterns and run
`product context FT-100 --depth 1 --measure`. Then read
`docs/features/FT-100-*.md` and parse its front-matter.

Assert:

1. The front-matter `bundle:` block contains `patterns: 2`.
2. Other `bundle:` metrics (depth-1-adrs, tcs, tokens-approx) are
   present and non-zero (existing FT-040 behaviour, unchanged).
3. Running `--measure` again with no graph changes produces the
   same `patterns:` count (idempotency).

## Formal specification

⟦Λ:Scenario⟧
Given FT-100 with `patterns: [PAT-A, PAT-B]`,
When the user runs `product context FT-100 --depth 1 --measure`,
Then the on-disk feature front-matter `bundle.patterns` reads 2,
And the measurement is idempotent across repeated runs against
  the same graph.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩