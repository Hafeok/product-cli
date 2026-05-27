---
id: TC-814
title: pattern_link_example_materialises_feature_patterns
type: scenario
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_814_pattern_link_example_materialises_feature_patterns
---

## Description

Compose a temp repo with pattern `PAT-001` (created via
`product pattern new`) and feature `FT-100`. Run
`product pattern link PAT-001 --example FT-100`.

Assert:

1. `docs/patterns/PAT-001-*.md` front-matter contains
   `examples: [FT-100]`.
2. `docs/features/FT-100-*.md` front-matter contains
   `patterns: [PAT-001]` (the bidirectional materialisation).
3. Reloading the graph via `parser::load_all` exposes both edges
   (graph membership, not just file presence).
4. The structured response from the CLI (JSON form, or MCP
   equivalent if the test goes through MCP) reports a `writes` array
   containing two paths and a `reciprocated` array with one entry —
   matching the FT-066 TC-787 shape for `product_feature_link`.

## Formal specification

⟦Λ:Scenario⟧
Given a repository containing pattern PAT-001 and feature FT-100,
When the user runs `product pattern link PAT-001 --example FT-100`,
Then `docs/patterns/PAT-001-*.md` carries `examples: [FT-100]` in
  its front-matter,
And `docs/features/FT-100-*.md` carries `patterns: [PAT-001]` in
  its front-matter,
And the loaded `KnowledgeGraph` exposes the edge in both
  directions,
And the structured JSON response includes both writes and one
  reciprocation entry.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
