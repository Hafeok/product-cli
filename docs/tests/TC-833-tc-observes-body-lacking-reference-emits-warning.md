---
id: TC-833
title: tc_observes_body_lacking_reference_emits_warning
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_833_tc_observes_body_lacking_reference_emits_warning
observes:
- stdout
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo with a scenario TC at phase 5 declaring
`observes: [file]` whose body text contains no mention of "file",
"disk", "wrote", or other configured synonyms. Run
`product graph check`.

Assert:

1. The output contains the new warning code naming the TC.
2. The warning is suppressible by adding the missing reference to
   the body — re-run after edit and the warning disappears.
3. Setting `[tc-observability].body-reference-severity = "error"`
   escalates the same finding to error severity with non-zero
   exit.

## Formal specification

⟦Λ:Scenario⟧
Given a phase-5 scenario TC with `observes: [file]` whose body
  never mentions file writes,
When the user runs `product graph check`,
Then the new body-reference warning is emitted naming the TC,
And the warning disappears once the body mentions the surface,
And escalation via `body-reference-severity = "error"` produces
  the same finding at error severity with non-zero exit.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩