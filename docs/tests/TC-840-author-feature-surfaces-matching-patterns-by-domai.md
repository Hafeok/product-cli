---
id: TC-840
title: author_feature_surfaces_matching_patterns_by_domain
type: scenario
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_840_author_feature_surfaces_matching_patterns_by_domain
---

## Description

Compose a temp repo with PAT-A (`domains: [api]`) and PAT-B
(`domains: [observability]`). Author a feature with `domains:
[api, observability]` via `product author feature`. Capture the
stdout / prompt-context dump from the session.

Assert:

1. The captured context contains a "Matching patterns" block (or
   equivalent header) listing both PAT-A and PAT-B.
2. The block includes each pattern's id, title, and status.
3. Setting `[patterns].suggest-domains = false` and rerunning
   suppresses the block entirely.
4. A feature with `domains: [unrelated]` (no overlap with any
   pattern) does not produce the block.

## Formal specification

⟦Λ:Scenario⟧
Given PAT-A (domains: [api]) and PAT-B (domains:
  [observability]),
When the user runs `product author feature` with feature
  domains [api, observability],
Then the session's prompt context includes a Matching patterns
  block listing both PATs,
And the block is suppressed by `[patterns].suggest-domains =
  false`,
And a feature with no overlapping domains does not get the
  block.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
