---
id: TC-831
title: tc_observes_missing_on_required_type_emits_error
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_831_tc_observes_missing_on_required_type_emits_error
observes:
- stdout
- exit-code
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo with `[tc-observability].required-from-phase
= 5` (the default). Author a scenario TC at phase 5 with no
`observes:` field. Run `product graph check`.

Assert:

1. The command exits with a non-zero code (mapped from the new
   error variant per ADR-013).
2. The output names the offending TC id.
3. The output references ADR-051 in the hint or detail.
4. The output lists the allowed surface vocabulary in the hint.

## Formal specification

⟦Λ:Scenario⟧
Given a phase-5 scenario TC with no `observes:` field,
When the user runs `product graph check`,
Then the command exits with a non-zero exit code,
And the diagnostic names the TC id, references ADR-051, and
  enumerates the allowed surface values.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩