---
id: TC-841
title: feature_link_pattern_writes_bidirectional
type: scenario
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_841_feature_link_pattern_writes_bidirectional
---

## Description

Compose a temp repo with PAT-001 and FT-100. Run `product
feature link FT-100 --pattern PAT-001`.

Assert:

1. `docs/features/FT-100-*.md` front-matter contains
   `patterns: [PAT-001]`.
2. `docs/patterns/PAT-001-*.md` front-matter contains
   `examples: [FT-100]`.
3. The structured JSON response carries the same `writes` and
   `reciprocated` shape FT-066 established (TC-787 generalised).
4. Re-running the command is idempotent — the response shows
   empty `writes` / `reciprocated`, files unchanged.

## Formal specification

⟦Λ:Scenario⟧
Given a repository with PAT-001 and FT-100,
When the user runs `product feature link FT-100 --pattern
  PAT-001`,
Then FT-100.patterns includes PAT-001,
And PAT-001.examples includes FT-100,
And the JSON response matches the FT-066 writes/reciprocated
  shape,
And a second identical invocation produces an empty response
  (idempotent).

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
