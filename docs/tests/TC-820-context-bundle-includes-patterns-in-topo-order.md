---
id: TC-820
title: context_bundle_includes_patterns_in_topo_order
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_820_context_bundle_includes_patterns_in_topo_order
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.4s
---

## Description

Compose a temp repo with three patterns: PAT-A (no requires), PAT-B
(`requires: [PAT-A]`), PAT-C (`requires: [PAT-A, PAT-B]`), and a
feature `FT-100` with `patterns: [PAT-C]`. Run `product context
FT-100 --depth 1` and capture stdout.

Assert:

1. The stdout contains a "## Patterns" heading.
2. Each pattern's body appears as a subsection.
3. PAT-A appears before PAT-B and PAT-B appears before PAT-C in the
   rendered order (topo over `requires:` edges).
4. The transitive prerequisites are included even though FT-100 only
   declares PAT-C directly (depth ≥ 1 walks `requires:`).

## Formal specification

⟦Λ:Scenario⟧
Given a repository with patterns PAT-A, PAT-B (requires PAT-A),
  PAT-C (requires PAT-A and PAT-B), and feature FT-100 (patterns:
  [PAT-C]),
When the user runs `product context FT-100 --depth 1`,
Then stdout contains a "## Patterns" heading,
And the body sections for PAT-A, PAT-B, PAT-C all appear,
And PAT-A precedes PAT-B which precedes PAT-C in the output text.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩