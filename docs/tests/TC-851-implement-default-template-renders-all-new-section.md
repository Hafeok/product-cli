---
id: TC-851
title: implement_default_template_renders_all_new_sections
type: scenario
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_851_implement_default_template_renders_all_new_sections
---

## Description

Regression guard for the default template. Compose a temp repo
with FT-100 citing PAT-A and two TCs with `observes:`. Run
`product implement FT-100 --dry-run` against the default
template.

Assert:

1. The bundle contains the "## Patterns" section.
2. The bundle contains observes lines adjacent to each TC body.
3. The bundle contains the ADR-051 hard-constraint line.
4. If any one of the three is missing, the TC fails — this is the
   contract that prevents the default template from drifting back
   into a pre-FT-074 shape.

## Formal specification

⟦Λ:Scenario⟧
Given the default implement template,
When the user runs `product implement FT-100 --dry-run`,
Then the bundle contains the Patterns section, the inline TC
  observes lines, and the ADR-051 hard-constraint line,
And missing any one of the three fails the TC.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
