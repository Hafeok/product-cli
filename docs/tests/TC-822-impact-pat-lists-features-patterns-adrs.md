---
id: TC-822
title: impact_pat_lists_features_patterns_adrs
type: scenario
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_822_impact_pat_lists_features_patterns_adrs
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo with PAT-A linked from feature FT-100 (via
`FT-100.patterns: [PAT-A]`), from pattern PAT-B (via
`PAT-B.requires: [PAT-A]`), and from ADR-050 (via
`PAT-A.adrs: [ADR-050]`). Run `product impact PAT-A` and capture
stdout.

Assert:

1. The output enumerates FT-100 (forward edge — feature uses PAT).
2. The output enumerates PAT-B (forward edge — pattern depends on
   PAT).
3. The output enumerates ADR-050 (downstream relation — pattern
   operationalises ADR).
4. The impact-tree format matches the existing `product impact
   FT-XXX` / `ADR-XXX` rendering style; the same JSON shape
   appears under `--format json`.

## Formal specification

⟦Λ:Scenario⟧
Given a repository where PAT-A is cited by FT-100, by
  PAT-B.requires, and cites ADR-050 in its adrs array,
When the user runs `product impact PAT-A`,
Then the stdout enumerates FT-100, PAT-B, and ADR-050 in the
  impact tree,
And the `--format json` output structures the same set under the
  existing impact schema.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩