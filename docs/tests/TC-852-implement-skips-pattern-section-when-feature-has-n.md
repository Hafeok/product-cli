---
id: TC-852
title: implement_skips_pattern_section_when_feature_has_none
type: scenario
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_852_implement_skips_pattern_section_when_feature_has_none
---

## Description

Compose a temp repo with FT-100 whose `patterns:` array is
empty. Run `product implement FT-100 --dry-run` and capture
the bundle.

Assert:

1. The "## Patterns" section header is **absent** from the
   bundle (no empty header rendered).
2. The bundle is otherwise well-formed — other sections (TCs,
   ADRs, hard constraints) appear normally.
3. The pipeline exits 0; an empty `patterns:` is not an error.

## Formal specification

⟦Λ:Scenario⟧
Given FT-100 with `patterns: []`,
When the user runs `product implement FT-100 --dry-run`,
Then the bundle does not render a "## Patterns" heading,
And the rest of the bundle is well-formed.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
