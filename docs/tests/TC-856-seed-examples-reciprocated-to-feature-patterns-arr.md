---
id: TC-856
title: seed_examples_reciprocated_to_feature_patterns_arrays
type: scenario
status: passing
validates:
  features:
  - FT-075
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_856_seed_examples_reciprocated_to_feature_patterns_arrays
last-run: 2026-05-27T15:36:27.790359954+00:00
last-run-duration: 2.6s
---

## Description

After applying the seed batch, read each example feature's
front-matter and confirm bidirectional materialisation per
ADR-050 / FT-070.

Assert:

1. `FT-066.patterns` includes every seed that cites FT-066 in
   its `examples:` list.
2. `FT-068.patterns` includes every seed that cites FT-068.
3. `FT-069.patterns` includes every seed that cites FT-069.
4. `FT-072.patterns` includes PAT-003 (since FT-072 is named as
   an example for the TC observability pattern).
5. The reloaded graph exposes the reciprocal edges from feature
   to pattern.

## Formal specification

⟦Λ:Scenario⟧
Given the F6 seed batch applied,
When each example feature's front-matter is read,
Then `feature.patterns` for FT-066, FT-068, FT-069, FT-072
  contains the seeds that listed those features in their
  `examples:`,
And the graph exposes the reciprocal edges.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩