---
id: TC-849
title: implement_bundle_contains_adr_051_hard_constraint_line
type: scenario
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_849_implement_bundle_contains_adr_051_hard_constraint_line
---

## Description

Compose a temp repo with FT-100. Run `product implement FT-100
--dry-run` and capture the bundle.

Assert:

1. The bundle contains a "Hard constraints" section (existing
   FT-068 structure).
2. Within that section, the verbatim ADR-051 reminder line
   appears: "Tests must assert against the surface(s) declared
   in each TC's `observes:` field..."
3. Mutating the implement prompt template to remove the line
   causes the regression-guard TC (TC-851) to fail; this TC
   independently asserts the line is present in the rendered
   output for the default template.

## Formal specification

⟦Λ:Scenario⟧
Given FT-100 and the default implement prompt template,
When the user runs `product implement FT-100 --dry-run`,
Then the bundle contains the "Hard constraints" section,
And that section contains the ADR-051 reminder line verbatim.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
