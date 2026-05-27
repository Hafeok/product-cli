---
id: TC-823
title: graph_check_requires_cycle_emits_error
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_823_graph_check_requires_cycle_emits_error
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo where PAT-A has `requires: [PAT-B]` and PAT-B
has `requires: [PAT-A]` (force the cycle by writing the files
directly — `product pattern link` would have refused via
TC-813's gate). Run `product graph check` and capture the exit
code and stdout.

Assert:

1. The exit code is non-zero (the new E-code allocated by this
   feature maps to a non-zero exit per ADR-013).
2. The stdout/stderr contains the new diagnostic code symbol and
   the substring `cycle`.
3. The reported cycle path includes both PAT-A and PAT-B.
4. The check terminates without panicking even on the cyclic
   topology.

## Formal specification

⟦Λ:Scenario⟧
Given a repository where PAT-A requires PAT-B and PAT-B requires
  PAT-A,
When the user runs `product graph check`,
Then the command exits with a non-zero code,
And the diagnostic output names the new requires-cycle error code,
And the cycle path mentions both PAT-A and PAT-B.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩