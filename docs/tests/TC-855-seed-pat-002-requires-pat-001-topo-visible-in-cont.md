---
id: TC-855
title: seed_pat_002_requires_pat_001_topo_visible_in_context
type: scenario
status: passing
validates:
  features:
  - FT-075
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_855_seed_pat_002_requires_pat_001_topo_visible_in_context
last-run: 2026-05-27T15:36:27.790359954+00:00
last-run-duration: 8.4s
---

## Description

After applying the seed batch (TC-854's precondition), invoke
`product context FT-066 --depth 1` against the repo (FT-066 is an
example for all three seeds, so all three appear in its bundle).

Assert:

1. The bundle stdout contains the body of PAT-001 and the body of
   PAT-002.
2. PAT-001 appears before PAT-002 in the rendered output (PAT-002
   requires PAT-001 — topo invariant).
3. PAT-003's position is consistent with its `requires:` (empty,
   so PAT-003 may appear anywhere relative to PAT-001/PAT-002 —
   the assertion is only on the PAT-001 < PAT-002 ordering).

## Formal specification

⟦Λ:Scenario⟧
Given the seed catalog applied and FT-066 citing all three
  seeds,
When the user runs `product context FT-066 --depth 1`,
Then PAT-001 appears before PAT-002 in the rendered bundle,
And every seed's body section is present.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩