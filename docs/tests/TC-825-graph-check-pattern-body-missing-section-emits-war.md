---
id: TC-825
title: graph_check_pattern_body_missing_section_emits_warning
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_825_graph_check_pattern_body_missing_section_emits_warning
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo with PAT-A (status `live`) whose markdown body
contains four of the five required H2 sections but omits
"Anti-patterns". Run `product graph check`.

Assert:

1. The stdout contains the new "pattern body missing section"
   warning code naming PAT-A and the missing heading.
2. The diagnostic carries a hint pointing at the configuration
   key `[patterns].body-sections`.
3. Setting `[patterns].body-severity = "error"` and re-running
   escalates to error severity and a non-zero exit (matching W030
   / E-class promotion behaviour from ADR-047).

## Formal specification

⟦Λ:Scenario⟧
Given PAT-A whose body lacks the "Anti-patterns" H2 heading,
When the user runs `product graph check` with default config,
Then a warning naming PAT-A and the missing heading is emitted,
And re-running with `[patterns].body-severity = "error"`
  produces the same finding at error severity with non-zero exit.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩