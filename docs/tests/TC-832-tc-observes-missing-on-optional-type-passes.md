---
id: TC-832
title: tc_observes_missing_on_optional_type_passes
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_832_tc_observes_missing_on_optional_type_passes
observes:
- exit-code
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo with `[tc-observability].required-from-phase
= 5` and two phase-5 TCs: one `type: invariant` and one `type:
property`, both without `observes:`. Run `product graph check`
and capture the exit code.

Assert:

1. The command exits with code 0 (no errors emitted for
   invariant or property TCs lacking `observes:`).
2. The output does not name either TC under the new error code.
3. Adding `observes: [exit-code]` to either TC continues to pass
   the gate (optionality holds in both directions).

## Formal specification

⟦Λ:Scenario⟧
Given a repository with phase-5 invariant and property TCs that
  lack `observes:`,
When the user runs `product graph check`,
Then the command exits 0,
And neither TC is named under the missing-observes error code.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩