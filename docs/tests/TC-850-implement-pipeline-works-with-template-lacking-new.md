---
id: TC-850
title: implement_pipeline_works_with_template_lacking_new_variables
type: scenario
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_850_implement_pipeline_works_with_template_lacking_new_variables
---

## Description

Compose a temp repo with FT-100 citing PAT-A and a custom
per-model template (FT-063) that omits the `{{patterns}}` and
`{{tc_observes_table}}` variables. Run `product implement
FT-100 --dry-run --target legacy-template`.

Assert:

1. The command exits 0 — the pipeline completes despite the
   missing variables.
2. The captured bundle does not contain the "## Patterns"
   section nor the inline observes lines (legacy compat).
3. Switching back to the default template causes both sections
   to reappear.

## Formal specification

⟦Λ:Scenario⟧
Given a custom template omitting the new variables,
When the user runs `product implement FT-100 --dry-run --target
  legacy-template`,
Then the command exits 0,
And the captured bundle lacks both new sections,
And switching to the default template restores them.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
