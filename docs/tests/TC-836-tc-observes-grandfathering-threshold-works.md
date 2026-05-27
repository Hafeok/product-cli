---
id: TC-836
title: tc_observes_grandfathering_threshold_works
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_836_tc_observes_grandfathering_threshold_works
observes:
- stdout
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo with a phase-5 scenario TC lacking
`observes:`. Verify the default config (`required-from-phase =
5`) flags it via stdout output. Edit `product.toml` to set
`required-from-phase = 99` and re-run `product graph check`,
capturing stdout each time.

Assert:

1. With the default config, the new error fires for the TC.
2. With `required-from-phase = 99`, the same TC passes (it is
   now below the threshold).
3. Setting `required-from-phase = 1` flags every required-for
   TC in the corpus regardless of phase.
4. The threshold change does not affect TCs whose type is not
   in `required-for-types`.

## Formal specification

⟦Λ:Scenario⟧
Given a phase-5 scenario TC with no `observes:`,
When the user toggles
  `[tc-observability].required-from-phase` between 5, 99, and 1,
Then graph check fires at thresholds ≤ TC phase and is silent
  at thresholds > TC phase,
And invariant / property TCs are unaffected by the threshold.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩